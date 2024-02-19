use bevy::prelude::*;

use crate::prelude::*;
use crate::{objects::objective::ObjectiveConfig, physics::PhysicsMaterialType};

/// Singleton that spawns birds with specified stats.
#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct InteractionConfig {
    pub separation_radius: f32,
    pub separation_acceleration: f32,
    pub cohesion_acceleration: f32,
    pub alignment_factor: f32,
    pub slow_factor: f32,
    pub chase: bool,
}
impl Default for InteractionConfig {
    fn default() -> Self {
        Self {
            separation_radius: 1.0,
            separation_acceleration: 0.0,
            cohesion_acceleration: 0.0,
            alignment_factor: 0.0,
            slow_factor: 0.0,
            chase: false,
        }
    }
}

/// Singleton that spawns birds with specified stats.
#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct Configs {
    // Specify which team the player controls.
    pub player_team: Team,
    pub visibility_radius: u16,
    pub fog_radius: u16,
    pub window_size: Vec2,
    pub cursor_sensitivity: f32,
    // Configs for each Zooid type.
    pub worker: Config,
    pub head: Config,
    pub food: Config,
}
impl Configs {
    pub fn get(&self, zooid: &Object) -> &Config {
        match zooid {
            Object::Worker => &self.worker,
            Object::Head => &self.head,
            Object::Food => &self.food,
        }
    }
}

/// Specifies stats per object type.
#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct Config {
    physics_material: PhysicsMaterialType,
    pub neighbor_radius: f32,
    pub alignment_factor: f32,
    pub obstacle_acceleration: f32,
    pub nav_flow_factor: f32,
    pub attack_velocity: f32,
    pub spawn_velocity: f32,
    pub waypoint: ObjectiveConfig,
    pub hit_radius: f32,
    pub death_speed: f32,

    // Interactions
    pub worker: InteractionConfig,
    pub head: InteractionConfig,
    pub food: InteractionConfig,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            physics_material: PhysicsMaterialType::Default,
            neighbor_radius: 10.0,
            alignment_factor: 0.1,
            obstacle_acceleration: 3.,
            nav_flow_factor: 1.,
            attack_velocity: 40.,
            spawn_velocity: 2.0,
            hit_radius: 10.0,
            death_speed: 9.0,
            waypoint: ObjectiveConfig::default(),
            worker: InteractionConfig::default(),
            head: InteractionConfig::default(),
            food: InteractionConfig::default(),
        }
    }
}
impl Config {
    pub fn get_interaction(&self, zooid: &Object) -> &InteractionConfig {
        match zooid {
            Object::Worker => &self.worker,
            Object::Head => &self.head,
            Object::Food => &self.food,
        }
    }

    /// Returns true if it's a hit.
    pub fn is_hit(&self, distance_squared: f32, velocity_squared: f32) -> bool {
        // info!("{}", velocity_squared);
        (distance_squared < self.hit_radius * self.hit_radius)
            && (velocity_squared > self.death_speed * self.death_speed)
    }
}
