use crate::prelude::*;
use bevy::prelude::*;

use super::{Configs, Object};

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

/// Represents the objective of the owning entity.
#[derive(Component, Default, Debug, Clone)]
pub enum Objective {
    /// Entity has no objective.
    #[default]
    None,
    /// Entity wants to follow the transform of another entity.
    #[allow(dead_code)]
    FollowEntity(Entity),
    /// Entity wants to move to a given position.
    MoveToPosition(Vec2),
}
impl Objective {
    pub fn update(
        mut query: Query<(&Self, &Object, &Transform, &Velocity, &mut NewVelocity)>,
        transforms: Query<&Transform>,
        configs: Res<Configs>,
    ) {
        for (follower, object, transform, velocity, mut new_velocity) in &mut query {
            let config = configs.get(object);
            new_velocity.0 +=
                follower.acceleration(&transforms, transform, velocity.0, &config.waypoint);
        }
    }

    fn acceleration_to_position(
        &self,
        velocity: Vec2,
        position: Vec2,
        target_position: Vec2,
        config: &ObjectiveConfig,
    ) -> Vec2 {
        let position_delta = target_position - position;
        let radius = config.repell_radius;
        let max_magnitude = config.max_acceleration;
        let dist_squared = position_delta.length_squared();
        let radius_squared = radius * radius;
        let magnitude = max_magnitude * (dist_squared / (radius_squared) - 1.);
        let slow_force = config.slow_factor
            * if dist_squared < radius_squared {
                Vec2::ZERO
            } else {
                -1.0 * velocity
            };
        position_delta.normalize_or_zero() * magnitude.clamp(-max_magnitude, max_magnitude)
            + slow_force
    }

    pub fn acceleration(
        &self,
        transforms: &Query<&Transform>,
        transform: &Transform,
        velocity: Vec2,
        config: &ObjectiveConfig,
    ) -> Vec2 {
        match self {
            &Objective::MoveToPosition(target_position) => self.acceleration_to_position(
                velocity,
                transform.translation.xy(),
                target_position,
                config,
            ),
            &Objective::FollowEntity(entity) => {
                let target_transform = transforms
                    .get(entity)
                    .expect(&format!("Invalid target ID: {:?}", entity));
                self.acceleration_to_position(
                    velocity,
                    transform.translation.xy(),
                    target_transform.translation.xy(),
                    config,
                )
            }
            &Objective::None => Vec2::ZERO,
        }
    }
}
