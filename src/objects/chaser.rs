use crate::prelude::*;
use bevy::prelude::*;

use super::{Configs, Object};

pub struct ChaserPlugin;
impl Plugin for ChaserPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ChaserConfig>()
            .add_systems(FixedUpdate, Chaser::update.in_set(SystemStage::PreCompute));
    }
}
#[derive(Resource, Debug, Clone, Reflect)]
#[reflect(Resource)]
pub struct ChaserConfig {
    pub max_acceleration: f32,
    pub repell_radius: f32,
    pub slow_factor: f32,
}
impl Default for ChaserConfig {
    fn default() -> Self {
        Self {
            max_acceleration: 0.0,
            repell_radius: 1.0,
            slow_factor: 0.0,
        }
    }
}

/// For storing the reference to the waypoint.
#[derive(Component, Default, Debug, Clone)]
pub struct Chaser {
    pub target_entity: Option<Entity>,
}
impl Chaser {
    #[allow(dead_code)]
    pub fn new(target_entity: Entity) -> Self {
        Self {
            target_entity: Some(target_entity),
        }
    }

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

    pub fn acceleration(
        &self,
        transforms: &Query<&Transform>,
        transform: &Transform,
        velocity: Vec2,
        config: &ChaserConfig,
    ) -> Vec2 {
        if let Some(entity) = self.target_entity {
            let targeting_transform = transforms
                .get(entity)
                .expect(&format!("Invalid target ID: {:?}", &self.target_entity));

            let position_delta = targeting_transform.translation.xy() - transform.translation.xy();
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
            return position_delta.normalize_or_zero()
                * magnitude.clamp(-max_magnitude, max_magnitude)
                + slow_force;
        }
        Vec2::ZERO
    }
}
