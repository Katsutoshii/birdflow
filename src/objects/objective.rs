use std::time::Duration;

use crate::prelude::*;
use bevy::prelude::*;
use rand::Rng;

pub struct ObjectivePlugin;
impl Plugin for ObjectivePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ObjectiveConfig>().add_systems(
            FixedUpdate,
            Objective::update.in_set(SystemStage::PreCompute),
        );
    }
}
#[derive(Resource, Debug, Clone, Reflect)]
#[reflect(Resource)]
pub struct ObjectiveConfig {
    pub max_acceleration: f32,
    pub repell_radius: f32,
    pub slow_factor: f32,
}
impl Default for ObjectiveConfig {
    fn default() -> Self {
        Self {
            max_acceleration: 0.0,
            repell_radius: 1.0,
            slow_factor: 0.0,
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

#[derive(Component, Debug, Clone)]
// Entity will attack nearest enemy in surrounding grid
pub struct AttackEntity {
    pub entity: Entity,
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
    /// Entity wants to move to a given position.
    #[allow(dead_code)]
    MoveToPosition(Vec2),
    #[allow(dead_code)]
    AttackEntity(AttackEntity),
}
impl Objective {
    /// Update acceleration from the current objective.
    pub fn update(
        mut query: Query<(&mut Self, &Object, &Transform, &Velocity, &mut Acceleration)>,
        transforms: Query<&Transform>,
        configs: Res<Configs>,
        navigation_grid: Res<EntityFlowGrid2>,
        time: Res<Time>,
    ) {
        for (mut objective, object, transform, velocity, mut acceleration) in &mut query {
            let config = configs.get(object);
            *acceleration += objective.acceleration(
                &transforms,
                transform,
                *velocity,
                &config.waypoint,
                &navigation_grid,
                &time,
            );
        }
    }

    /// Returns acceleration towards a given position.
    fn acceleration_to_position(
        velocity: Velocity,
        position: Vec2,
        target_position: Vec2,
        config: &ObjectiveConfig,
    ) -> Acceleration {
        let position_delta = target_position - position;
        let radius = config.repell_radius;
        let max_magnitude = config.max_acceleration;
        let dist_squared = position_delta.length_squared();
        let radius_squared = radius * radius;
        let magnitude = max_magnitude * (dist_squared / (radius_squared) - 1.);
        Acceleration(
            position_delta.normalize_or_zero() * magnitude.clamp(-max_magnitude, max_magnitude),
        ) + config.slow_force(velocity, position, target_position)
    }

    // Returns acceleration for following an entity.
    pub fn accelerate_to_entity(
        entity: Entity,
        transform: &Transform,
        transforms: &Query<&Transform>,
        config: &ObjectiveConfig,
        velocity: Velocity,
        flow_grid: &EntityFlowGrid2,
    ) -> Acceleration {
        let target_transform = transforms.get(entity);
        let target_transform = match target_transform {
            Ok(transform) => transform,
            Err(_) => return Acceleration::ZERO,
        };
        if let Some(flow_grid) = flow_grid.get(&entity) {
            let target_cell = flow_grid.to_rowcol(target_transform.translation.xy());
            let target_cell_position = flow_grid.to_world_position(target_cell);
            flow_grid.flow_acceleration5(transform.translation.xy())
                + config.slow_force(velocity, transform.translation.xy(), target_cell_position)
        } else {
            error!("Missing entity: {:?}", entity);
            Acceleration::ZERO
        }
    }

    // Returns acceleration for this objective.
    pub fn acceleration(
        &mut self,
        transforms: &Query<&Transform>,
        transform: &Transform,
        velocity: Velocity,
        config: &ObjectiveConfig,
        navigation_grid: &EntityFlowGrid2,
        time: &Time,
    ) -> Acceleration {
        match self {
            Self::MoveToPosition(target_position) => Self::acceleration_to_position(
                velocity,
                transform.translation.xy(),
                *target_position,
                config,
            ),
            Self::FollowEntity(entity) => Self::accelerate_to_entity(
                *entity,
                transform,
                transforms,
                config,
                velocity,
                navigation_grid,
            ),
            Self::AttackEntity(AttackEntity { entity, cooldown }) => {
                cooldown.tick(time.delta());
                if cooldown.finished() {
                    cooldown.set_duration(Self::attack_cooldown());
                    let target_transform = transforms.get(*entity);
                    let target_transform = match target_transform {
                        Ok(transform) => transform,
                        Err(_) => return Acceleration::ZERO,
                    };
                    let delta = target_transform.translation.xy() - transform.translation.xy();
                    Acceleration(delta.normalize() * 1000.0)
                } else {
                    Self::accelerate_to_entity(
                        *entity,
                        transform,
                        transforms,
                        config,
                        velocity,
                        navigation_grid,
                    )
                }
            }
            Self::None => Acceleration::ZERO,
        }
    }

    pub fn get_followed_entity(&self) -> Option<Entity> {
        match self {
            Self::AttackEntity(AttackEntity {
                entity,
                cooldown: _,
            }) => Some(*entity),
            Self::FollowEntity(entity) => Some(*entity),
            _ => None,
        }
    }
    // Gets a random attack cooldown.
    pub fn attack_cooldown() -> Duration {
        Duration::from_millis(rand::thread_rng().gen_range(800..1200))
    }

    // Gets a random attack cooldown.
    pub fn attack_delay() -> Duration {
        Duration::from_millis(rand::thread_rng().gen_range(0..100))
    }

    /// Given an objective, get the next one (if there should be a next one, else None).
    pub fn next(&self, closest_enemy_entity: Option<Entity>) -> Option<Self> {
        match *self {
            Self::None | Self::FollowEntity(_) => closest_enemy_entity.map(|entity| {
                Self::AttackEntity(AttackEntity {
                    entity,
                    cooldown: Timer::from_seconds(
                        Self::attack_delay().as_secs_f32(),
                        TimerMode::Repeating,
                    ),
                })
            }),
            Self::MoveToPosition(_) => None,
            Self::AttackEntity(_) => None,
        }
    }
}
