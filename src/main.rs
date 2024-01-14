use bevy::{ecs::schedule::SystemSetConfigs, prelude::*};
use bevy_editor_pls::prelude::*;

pub mod aabb;
pub mod camera;
pub mod grid;
pub mod inputs;
pub mod meshes;
pub mod objects;
pub mod physics;
pub mod raycast;
pub mod scene;
pub mod selector;
pub mod window;
pub mod zindex;

pub mod prelude {
    pub use crate::{
        aabb::Aabb2,
        camera::MainCamera,
        grid::{
            EntityFlow, EntityGridEvent, EntitySet, Grid2, Grid2Plugin, GridEntity, GridSize,
            GridSpec, Obstacle, RowCol, RowColDistance,
        },
        inputs::{ControlAction, ControlEvent, InputState},
        meshes,
        objects::{Config, Configs, Object, Objective, Team},
        physics::{Acceleration, PhysicsBundle, PhysicsMaterial, PhysicsMaterialType, Velocity},
        raycast::{RaycastEvent, RaycastTarget},
        selector::Selected,
        window, zindex, SystemStage,
    };
}

use prelude::*;

/// Stage of computation
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum SystemStage {
    Spawn,
    PreCompute,
    Compute,
    Apply,
    PostApply,
    Despawn,
}
impl SystemStage {
    pub fn get_config() -> SystemSetConfigs {
        (
            Self::Spawn,
            Self::PreCompute,
            Self::Compute,
            Self::Apply,
            Self::PostApply,
            Self::Despawn,
        )
            .chain()
    }
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(AssetPlugin {
                    watch_for_changes_override: Some(true),
                    ..default()
                })
                .set(window::custom_plugin()),
            inputs::InputActionPlugin,
            EditorPlugin::default(),
            grid::GridPlugin,
            objects::ObjectsPlugin,
            scene::LoadableScenePlugin,
            selector::SelectorPlugin,
            raycast::RaycastPlugin,
            camera::CameraPlugin,
            physics::PhysicsPlugin,
        ))
        .add_systems(Startup, startup)
        .add_systems(
            FixedUpdate,
            window::resize_window.in_set(SystemStage::Spawn),
        )
        .run();
}

fn startup(mut commands: Commands) {
    commands.spawn((TextBundle::from_section(
        [
            "  Controls:",
            "    Create your spawner: enter",
            "    Move camera: move mouse to border",
            "    Move waypoint: right click",
            "    Spawn zooids: 'z'",
            "    Despawn zooids: 'd'",
            "    Save scene: 's'",
            "    Open editor: 'e'",
            "    -",
        ]
        .join("\n"),
        TextStyle {
            font_size: 18.0,
            ..default()
        },
    )
    .with_style(Style {
        align_self: AlignSelf::FlexEnd,
        ..default()
    }),));
}
