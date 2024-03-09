use crate::prelude::*;
use bevy::prelude::*;

pub use self::{
    commands::{ObjectCommands, ObjectSpec},
    config::{
        InteractionConfig, InteractionConfigs, ObjectConfig, ObjectConfigs, TestInteractionConfigs,
    },
    damage::{DamageEvent, Health},
    object::Object,
    objective::{Objective, ObjectiveConfig, ObjectiveDebugger, Objectives},
};
use self::{
    damage::DamagePlugin, object::ObjectPlugin, objective::ObjectivePlugin,
    plankton::PlanktonPlugin, zooid_head::ZooidHeadPlugin, zooid_worker::ZooidWorkerPlugin,
};

/// Plugin for running zooids simulation.
pub struct ObjectsPlugin;
impl Plugin for ObjectsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ObjectivePlugin,
            ZooidHeadPlugin,
            ZooidWorkerPlugin,
            PlanktonPlugin,
            ObjectPlugin,
            DamagePlugin,
        ))
        .init_resource::<ZooidAssets>()
        .configure_sets(FixedUpdate, SystemStage::get_config());
    }
}

mod commands;
mod config;
mod damage;
mod object;
mod objective;
mod plankton;
mod zooid_head;
mod zooid_worker;

/// Enum to specify the team of the given object.
#[derive(Component, Default, Debug, PartialEq, Eq, Reflect, Clone, Copy, Hash, clap::ValueEnum)]
#[reflect(Component)]
#[repr(u8)]
pub enum Team {
    #[default]
    None = 0,
    Blue = 1,
    Red = 2,
}
impl Team {
    /// Number of teams.
    pub const COUNT: usize = 3;

    pub const BRIGHT_SEA_GREEN: Color = Color::rgb(0.18 + 0.2, 0.55 + 0.2, 0.34 + 0.2);
    pub const BRIGHT_TEAL: Color = Color::rgb(0.1, 0.5 + 0.2, 0.5 + 0.2);

    pub const ALL: [Self; Self::COUNT] = [Self::None, Self::Blue, Self::Red];
    pub const COLORS: [Color; Self::COUNT] =
        [Self::BRIGHT_SEA_GREEN, Self::BRIGHT_TEAL, Color::TOMATO];
}

#[derive(Default, Clone)]
pub struct TeamMaterials {
    pub primary: Handle<ColorMaterial>,
    pub background: Handle<ColorMaterial>,
}
impl TeamMaterials {
    pub fn new(color: Color, assets: &mut Assets<ColorMaterial>) -> Self {
        Self {
            primary: assets.add(ColorMaterial::from(color)),
            background: assets.add(ColorMaterial::from(color.with_a(0.3))),
        }
    }
}

/// Handles to common zooid assets.
#[derive(Resource)]
pub struct ZooidAssets {
    pub mesh: Handle<Mesh>,
    team_materials: Vec<TeamMaterials>,
}
impl ZooidAssets {
    fn get_team_material(&self, team: Team) -> TeamMaterials {
        self.team_materials.get(team as usize).unwrap().clone()
    }
}
impl FromWorld for ZooidAssets {
    fn from_world(world: &mut World) -> Self {
        let mesh = {
            let mut meshes = world.get_resource_mut::<Assets<Mesh>>().unwrap();
            meshes.add(Mesh::from(Circle::default()))
        };
        let mut materials = world.get_resource_mut::<Assets<ColorMaterial>>().unwrap();
        Self {
            mesh,
            team_materials: Team::COLORS
                .iter()
                .map(|color| TeamMaterials::new(*color, &mut materials))
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_update() {}
}
