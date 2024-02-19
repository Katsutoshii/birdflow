use std::{f32::consts::PI, time::Duration};

use crate::prelude::*;
use bevy::{prelude::*, text::Text2dBounds};
use rand::Rng;

use super::object::AttackEvent;

pub struct ObjectivePlugin;
impl Plugin for ObjectivePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ObjectiveConfig>().add_systems(
            FixedUpdate,
            (
                Objectives::update
                    .in_set(SystemStage::PreCompute)
                    .after(NavigationGrid2::update_waypoints),
                ObjectiveDebugger::update
                    .in_set(SystemStage::PreCompute)
                    .after(Objectives::update),
            ),
        );
    }
}
#[derive(Resource, Debug, Clone, Reflect)]
#[reflect(Resource)]
pub struct ObjectiveConfig {
    pub max_acceleration: f32,
    pub repell_radius: f32,
    pub slow_factor: f32,
    pub attack_radius: f32,
}
impl Default for ObjectiveConfig {
    fn default() -> Self {
        Self {
            max_acceleration: 0.0,
            repell_radius: 1.0,
            slow_factor: 0.0,
            attack_radius: 32.0,
        }
    }
}
impl ObjectiveConfig {
    /// Apply a slowing force against current velocity when near the goal.
    /// Also, undo some of the acceleration force when near the goal.
    pub fn slow_force(
        &self,
        velocity: Velocity,
        position: Vec2,
        target_position: Vec2,
        flow_acceleration: Acceleration,
    ) -> Acceleration {
        let position_delta = target_position - position;
        let dist_squared = position_delta.length_squared();
        let radius = self.repell_radius;
        let radius_squared = radius * radius;

        //  When within radius, this is negative
        let radius_diff = (dist_squared - radius_squared) / radius_squared;
        Acceleration(
            self.slow_factor
                * if dist_squared < radius_squared {
                    -1.0 * velocity.0
                } else {
                    Vec2::ZERO
                }
                + flow_acceleration.0 * radius_diff.clamp(-1., 0.),
        )
    }
}

#[derive(Debug, Clone)]
// Entity will attack nearest enemy in surrounding grid
pub struct AttackEntity {
    pub entity: Entity,
    pub frame: u16,
    pub cooldown: Timer,
}

/// Represents the objective of the owning entity.
#[derive(Component, Default, Debug, Clone, PartialEq)]
pub enum Objective {
    /// Entity has no objective.
    #[default]
    None,
    /// Entity wants to follow the transform of another entity.
    FollowEntity(Entity),
    /// Attack Entity
    AttackEntity {
        entity: Entity,
        frame: u16,
        cooldown: Timer,
    },
}
impl Objective {
    /// Given an objective, get the next one (if there should be a next one, else None).
    pub fn try_attacking(&self, entity: Entity) -> Option<Self> {
        match self {
            Self::None | Self::FollowEntity(_) => Some(Self::AttackEntity {
                entity,
                frame: 0,
                cooldown: Timer::from_seconds(
                    Self::attack_delay().as_secs_f32(),
                    TimerMode::Repeating,
                ),
            }),
            Self::AttackEntity {
                entity: _,
                frame: _,
                cooldown: _,
            } => None,
        }
    }

    /// Gets a random attack delay.
    pub fn attack_delay() -> Duration {
        Duration::from_millis(rand::thread_rng().gen_range(0..300))
    }

    /// Gets a random attack cooldown.
    pub fn attack_cooldown() -> Duration {
        Duration::from_millis(rand::thread_rng().gen_range(0..1200))
    }

    /// Resolves an objective.
    pub fn resolve(
        &mut self,
        transform: &Transform,
        query: &Query<(&Transform, &Velocity)>,
        time: &Time,
        config: &ObjectiveConfig,
    ) -> ResolvedObjective {
        match self {
            Self::None => ResolvedObjective::None,
            Self::FollowEntity(entity) => {
                if let Ok((other_transform, _other_velocity)) = query.get(*entity) {
                    ResolvedObjective::FollowEntity {
                        entity: *entity,
                        position: other_transform.translation.xy(),
                    }
                } else {
                    ResolvedObjective::None
                }
            }
            Self::AttackEntity {
                entity,
                frame,
                cooldown,
            } => {
                cooldown.tick(time.delta());
                if let Ok((other_transform, other_velocity)) = query.get(*entity) {
                    let position = transform.translation.xy();
                    let other_position = other_transform.translation.xy();
                    let target_position = other_position + other_velocity.0;
                    let delta = target_position - position;
                    if delta.length_squared() < config.attack_radius * config.attack_radius
                        && cooldown.finished()
                    {
                        cooldown.set_duration(Self::attack_cooldown());
                        *frame = 3;
                    }
                    if *frame > 0 {
                        *frame -= 1;
                    }
                    ResolvedObjective::AttackEntity {
                        entity: *entity,
                        position,
                        target_position,
                        frame: *frame,
                    }
                } else {
                    ResolvedObjective::None
                }
            }
        }
    }

    pub fn get_followed_entity(&self) -> Option<Entity> {
        match self {
            Self::AttackEntity {
                entity,
                frame: _,
                cooldown: _,
            } => Some(*entity),
            Self::FollowEntity(entity) => Some(*entity),
            _ => None,
        }
    }
}
/// Represents the objectives of the owning entity.
/// The stack always has Objective::None at the bottom.
#[derive(Component, Debug, Clone)]
pub struct Objectives(Vec<Objective>);
impl Default for Objectives {
    fn default() -> Self {
        Self(vec![Objective::None])
    }
}
impl Objectives {
    /// Construct an objective with default to None (idle).
    pub fn new(objective: Objective) -> Self {
        Self(vec![Objective::None, objective])
    }
    /// Get the last objective.
    pub fn last(&self) -> &Objective {
        unsafe { self.0.get_unchecked(self.0.len() - 1) }
    }
    /// Get the last objective.
    pub fn last_mut(&mut self) -> &mut Objective {
        let i = self.0.len() - 1;
        unsafe { self.0.get_unchecked_mut(i) }
    }
    /// Resets the objectives.
    pub fn clear(&mut self) {
        *self = Self::default();
    }
    /// Push an objective on the stack.
    pub fn push(&mut self, objective: Objective) {
        self.0.push(objective)
    }
    /// Pop an objective, but only if it's not the bottom None objective.
    pub fn pop(&mut self) -> Option<Objective> {
        if self.0.len() > 1 {
            self.0.pop()
        } else {
            None
        }
    }

    // Start attacking
    pub fn start_attacking(&mut self, attack_event: &AttackEvent) {
        if let Some(objective) = self.last().try_attacking(attack_event.entity) {
            self.push(objective);
        }
    }

    /// Update acceleration from the current objective.
    pub fn update(
        mut query: Query<(&mut Self, &Object, &Transform, &Velocity, &mut Acceleration)>,
        others: Query<(&Transform, &Velocity)>,
        configs: Res<Configs>,
        grid_spec: Res<GridSpec>,
        navigation_grid: Res<NavigationGrid2>,
        obstacles_grid: Res<Grid2<Obstacle>>,
        time: Res<Time>,
    ) {
        for (mut objectives, object, transform, velocity, mut acceleration) in &mut query {
            let config = configs.objects.get(object).unwrap();
            let resolved = objectives.resolve(transform, &others, &time, &config.waypoint);
            *acceleration +=
                resolved.acceleration(transform, *velocity, config, &grid_spec, &navigation_grid);
            let current_acceleration = *acceleration;
            *acceleration += obstacles_grid.obstacles_acceleration(
                transform.translation.xy(),
                *velocity,
                current_acceleration,
            ) * config.obstacle_acceleration;
        }
    }

    /// Resolve the entity references for the objective and store them in ResolvedObjective.
    /// If there are invalid entity references (deleted entities), remove those objectives.
    pub fn resolve(
        &mut self,
        transform: &Transform,
        query: &Query<(&Transform, &Velocity)>,
        time: &Time,
        config: &ObjectiveConfig,
    ) -> ResolvedObjective {
        while self.last() != &Objective::None {
            let resolved = self.last_mut().resolve(transform, query, time, config);
            if resolved != ResolvedObjective::None {
                return resolved;
            }
            self.0.pop();
        }
        ResolvedObjective::None
    }
}

/// Represents the objective of the owning entity.
#[derive(Component, Default, Debug, Clone, PartialEq)]
pub enum ResolvedObjective {
    /// Entity has no objective.
    #[default]
    None,
    /// Entity wants to follow the transform of another entity.
    FollowEntity { entity: Entity, position: Vec2 },
    /// Attack Entity
    AttackEntity {
        entity: Entity,
        position: Vec2,
        target_position: Vec2,
        frame: u16,
    },
}
impl ResolvedObjective {
    // Returns acceleration for this objective.
    pub fn acceleration(
        &self,
        transform: &Transform,
        velocity: Velocity,
        config: &ObjectConfig,
        grid_spec: &GridSpec,
        navigation_grid: &NavigationGrid2,
    ) -> Acceleration {
        let position = transform.translation.xy();
        match self {
            Self::FollowEntity {
                entity: _,
                position: target_position,
            } => Self::accelerate_to_position(
                position,
                *target_position,
                config,
                velocity,
                grid_spec,
                navigation_grid,
                /*slow_factor=*/ 1.0,
            ),
            Self::AttackEntity {
                entity: _,
                position,
                target_position,
                frame,
            } => {
                let delta = *target_position - *position;
                if *frame > 0 {
                    Acceleration(delta.normalize() * config.attack_velocity)
                } else {
                    Self::accelerate_to_position(
                        *position,
                        *target_position,
                        config,
                        velocity,
                        grid_spec,
                        navigation_grid,
                        /*slow_factor=*/ 0.5,
                    ) + Acceleration(delta.normalize() * 0.0)
                }
            }
            Self::None => {
                // If no objective, slow down and circle about.
                let reduce_velocity = velocity.0 / 2.;
                Acceleration(Mat2::from_angle(PI / 16.) * reduce_velocity - reduce_velocity)
            }
        }
    }

    // Returns acceleration for following an entity.
    pub fn accelerate_to_position(
        position: Vec2,
        target_position: Vec2,
        config: &ObjectConfig,
        velocity: Velocity,
        grid_spec: &GridSpec,
        navigation_grid: &NavigationGrid2,
        slow_factor: f32,
    ) -> Acceleration {
        let target_cell = grid_spec.to_rowcol(target_position);
        if let Some(nav) = navigation_grid.get(&target_cell) {
            let target_cell_position = nav.grid.to_world_position(target_cell);
            let flow_acceleration = nav.grid.flow_acceleration5(position) * config.nav_flow_factor;
            flow_acceleration
                + config.waypoint.slow_force(
                    velocity,
                    position,
                    target_cell_position,
                    flow_acceleration,
                ) * slow_factor
        } else {
            // TODO figure out why this logs sometimes. Commenting out to avoid spamming.
            // warn!(
            //     "Missing target_cell. This is okay if it's only for one frame. {:?}",
            //     target_cell
            // );
            Acceleration::ZERO
        }
    }
}

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct ObjectiveDebugger;
impl ObjectiveDebugger {
    #[allow(dead_code)]
    pub fn bundle(self) -> impl Bundle {
        info!("ObjectiveDebugger::bundle");
        (
            Text2dBundle {
                text: Text {
                    sections: vec![TextSection::new(
                        "Objective",
                        TextStyle {
                            font_size: 18.0,
                            ..default()
                        },
                    )],
                    alignment: TextAlignment::Center,
                    ..default()
                },
                text_2d_bounds: Text2dBounds {
                    // Wrap text in the rectangle
                    size: Vec2::new(1., 1.),
                },
                // ensure the text is drawn on top of the box
                transform: Transform::from_translation(Vec3::Z).with_scale(Vec3::new(0.1, 0.1, 1.)),
                ..default()
            },
            self,
        )
    }

    #[allow(dead_code)]
    pub fn update(
        mut query: Query<(&mut Text, &Parent), With<Self>>,
        objectives: Query<&Objective, Without<Self>>,
    ) {
        for (mut text, parent) in query.iter_mut() {
            let objective = objectives.get(parent.get()).unwrap();
            *text = Text::from_sections(vec![TextSection::new(
                format!("{:?}", objective),
                TextStyle {
                    font_size: 18.0,
                    ..default()
                },
            )]);
        }
    }
}
