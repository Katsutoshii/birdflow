use bevy::prelude::*;

use crate::objects::objective::ObjectiveConfig;

use super::{Object, Team};

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
    // Configs for each Zooid type.
    pub worker: Config,
    pub head: Config,
    pub food: Config,
}
impl Configs {
    pub fn get(&self, zooid: &Object) -> &Config {
        match zooid {
            Object::Worker(_) => &self.worker,
            Object::Head => &self.head,
            Object::Food => &self.food,
        }
    }
}

/// Singleton that spawns birds with specified stats.
#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct Config {
    pub max_velocity: f32,
    pub neighbor_radius: f32,
    pub alignment_factor: f32,
    pub velocity_smoothing: f32,
    pub spawn_velocity: f32,
    pub obstacle_repel: f32,
    pub waypoint: ObjectiveConfig,

    // Interactions
    pub worker: InteractionConfig,
    pub head: InteractionConfig,
    pub food: InteractionConfig,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            max_velocity: 10.0,
            neighbor_radius: 10.0,
            alignment_factor: 0.1,
            velocity_smoothing: 0.5,
            spawn_velocity: 2.0,
            obstacle_repel: 1.0,
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
            Object::Worker(_) => &self.worker,
            Object::Head => &self.head,
            Object::Food => &self.food,
        }
    }
}
