use crate::prelude::*;
use bevy::prelude::*;

pub use self::{
    config::{Config, Configs, InteractionConfig},
    objective::Objective,
};
use self::{
    food::FoodPlugin, object::ObjectPlugin, objective::ObjectivePlugin,
    zooid_head::ZooidHeadPlugin, zooid_worker::ZooidWorkerPlugin,
};

/// Plugin for running zooids simulation.
pub struct ObjectsPlugin;
impl Plugin for ObjectsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ObjectivePlugin,
            ZooidHeadPlugin,
            ZooidWorkerPlugin,
            FoodPlugin,
            ObjectPlugin,
        ))
        .register_type::<Vec2>()
        .register_type::<Configs>()
        .register_type::<Config>()
        .register_type::<Team>()
        .register_type::<InteractionConfig>()
        .init_resource::<ZooidAssets>()
        .configure_sets(FixedUpdate, SystemStage::get_config());
    }
}

mod config;
mod food;
mod object;
mod objective;
mod zooid_head;
mod zooid_worker;

pub use object::Object;

/// Enum to specify the team of the given object.
#[derive(Component, Default, Debug, PartialEq, Eq, Reflect, Clone, Copy, Hash)]
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
            background: assets.add(ColorMaterial::from(color.with_a(0.2))),
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
            meshes.add(Mesh::from(shape::Circle::default()))
        };
        let mut materials = world.get_resource_mut::<Assets<ColorMaterial>>().unwrap();
        Self {
            mesh,
            team_materials: vec![
                // Team::None
                TeamMaterials::new(Color::SEA_GREEN, &mut materials),
                // Team::Blue
                TeamMaterials::new(Color::TEAL, &mut materials),
                // Team::Red
                TeamMaterials::new(Color::TOMATO, &mut materials),
            ],
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_update() {}
}
