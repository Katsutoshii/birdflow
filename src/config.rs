use bevy::prelude::*;
use bevy::utils::HashMap;

use crate::objects::{InteractionConfig, ObjectConfig};
use crate::prelude::*;

pub struct ConfigPlugin;
impl Plugin for ConfigPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Vec2>()
            .register_type::<Configs>()
            .register_type::<ObjectConfig>()
            .register_type::<InteractionConfig>()
            .register_type::<Team>();
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

    // Configs per object type.
    pub objects: HashMap<Object, ObjectConfig>,
}
