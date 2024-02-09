use std::{f32::consts::PI, time::Duration};

use crate::prelude::*;
use bevy::{prelude::*, text::Text2dBounds};
use rand::Rng;

pub struct ObjectivePlugin;
impl Plugin for ObjectivePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ObjectiveConfig>().add_systems(
            FixedUpdate,
            (
                Objective::update.in_set(SystemStage::PreCompute),
                ObjectiveDebugger::update
                    .in_set(SystemStage::PreCompute)
                    .after(Objective::update),
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
    /// Apply a slowing force.
    pub fn slow_force(
        &self,
        velocity: Velocity,
        position: Vec2,
        target_position: Vec2,
    ) -> Acceleration {
        let position_delta = target_position - position;
        let dist_squared = position_delta.length_squared();
        let radius = self.repell_radius;
        let radius_squared = radius * radius;
        Acceleration(
            self.slow_factor
                * if dist_squared < radius_squared {
                    -1.0 * velocity.0
                } else {
                    Vec2::ZERO
                },
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
#[derive(Component, Default, Debug, Clone)]
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
    /// Update acceleration from the current objective.
    pub fn update(
        mut query: Query<(&mut Self, &Object, &Transform, &Velocity, &mut Acceleration)>,
        others: Query<(&Transform, &Velocity)>,
        configs: Res<Configs>,
        navigation_grid: Res<EntityFlowGrid2>,
        obstacles_grid: Res<Grid2<Obstacle>>,
        time: Res<Time>,
    ) {
        for (mut objective, object, transform, velocity, mut acceleration) in &mut query {
            let config = configs.get(object);
            if let Some(resolved) = objective.resolve(transform, &others, &time, &config.waypoint) {
                *acceleration +=
                    resolved.acceleration(transform, *velocity, &config.waypoint, &navigation_grid);
                let current_acceleration = *acceleration;
                *acceleration += obstacles_grid.obstacles_acceleration(
                    transform.translation.xy(),
                    *velocity,
                    current_acceleration,
                ) * 3.;
            } else {
                *objective = Self::None;
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
    /// Gets a random attack cooldown.
    pub fn attack_cooldown() -> Duration {
        Duration::from_millis(rand::thread_rng().gen_range(0..1200))
    }

    /// Gets a random attack delay.
    pub fn attack_delay() -> Duration {
        Duration::from_millis(rand::thread_rng().gen_range(0..300))
    }

    /// Given an objective, get the next one (if there should be a next one, else None).
    pub fn next(&self, event: &Option<CreateWaypointEvent>) -> Option<Self> {
        match self {
            Self::None | Self::FollowEntity(_) => event.as_ref().map(|event| Self::AttackEntity {
                entity: event.entity,
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

    /// Resolve the entity references for the objective and store them in ResolvedObjective.
    /// If there are invalid entity references (deleted entities), return None.
    pub fn resolve(
        &mut self,
        transform: &Transform,
        query: &Query<(&Transform, &Velocity)>,
        time: &Time,
        config: &ObjectiveConfig,
    ) -> Option<ResolvedObjective> {
        match self {
            Self::None => Some(ResolvedObjective::None),
            Self::FollowEntity(entity) => {
                if let Ok((other_transform, _other_velocity)) = query.get(*entity) {
                    Some(ResolvedObjective::FollowEntity {
                        entity: *entity,
                        position: other_transform.translation.xy(),
                    })
                } else {
                    None
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
                    Some(ResolvedObjective::AttackEntity {
                        entity: *entity,
                        position,
                        target_position,
                        frame: *frame,
                    })
                } else {
                    None
                }
            }
        }
    }
}

/// Represents the objective of the owning entity.
#[derive(Component, Default, Debug, Clone)]
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
        config: &ObjectiveConfig,
        navigation_grid: &EntityFlowGrid2,
    ) -> Acceleration {
        let position = transform.translation.xy();
        match self {
            Self::FollowEntity {
                entity,
                position: target_position,
            } => Self::accelerate_to_entity(
                *entity,
                position,
                *target_position,
                config,
                velocity,
                navigation_grid,
            ),
            Self::AttackEntity {
                entity,
                position,
                target_position,
                frame,
            } => {
                let delta = *target_position - *position;
                if *frame > 0 {
                    Acceleration(delta.normalize() * 40.0)
                } else {
                    Self::accelerate_to_entity(
                        *entity,
                        *position,
                        *target_position,
                        config,
                        velocity,
                        navigation_grid,
                    ) + Acceleration(delta.normalize() * 0.0)
                }
            }
            Self::None => {
                // If no objective, slow down and circle about.
                let half_velocity = velocity.0 / 2.;
                Acceleration(Mat2::from_angle(PI / 4.) * half_velocity - half_velocity)
            }
        }
    }

    // Returns acceleration for following an entity.
    pub fn accelerate_to_entity(
        entity: Entity,
        position: Vec2,
        target_position: Vec2,
        config: &ObjectiveConfig,
        velocity: Velocity,
        navigation_grid: &EntityFlowGrid2,
    ) -> Acceleration {
        if let Some(flow_grid) = navigation_grid.get(&entity) {
            let target_cell = flow_grid.to_rowcol(target_position);
            let target_cell_position = flow_grid.to_world_position(target_cell);
            flow_grid.flow_acceleration5(position)
                + config.slow_force(velocity, position, target_cell_position)
        } else {
            warn!(
                "Missing entity. This is okay if it's only for one frame. Entity: {:?}",
                entity
            );
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
