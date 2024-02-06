use bevy::prelude::*;

pub mod aabb;
pub mod camera;
pub mod cursor;
pub mod effects;
pub mod grid;
pub mod inputs;
pub mod meshes;
pub mod objects;
pub mod physics;
pub mod raycast;
pub mod scene;
pub mod selector;
pub mod stages;
pub mod waypoint;
pub mod window;
pub mod zindex;

pub mod prelude {
    pub use crate::{
        aabb::Aabb2,
        camera::{CameraController, CameraMoveEvent, MainCamera},
        cursor::Cursor,
        effects,
        grid::{
            CreateWaypointEvent, EntityFlowGrid2, EntityGridEvent, EntitySet, Grid2, Grid2Plugin,
            GridEntity, GridSize, GridSpec, Obstacle, RowCol, RowColDistance,
        },
        inputs::{ControlAction, ControlEvent, InputState},
        meshes,
        objects::{Config, Configs, Health, Object, Objective, Objectives, Team},
        physics::{Acceleration, PhysicsBundle, PhysicsMaterial, PhysicsMaterialType, Velocity},
        raycast::{RaycastEvent, RaycastTarget},
        selector::Selected,
        stages::SystemStage,
        waypoint::Waypoint,
        window, zindex,
    };
}

use prelude::*;

use bevy_hanabi::prelude::*;

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
            grid::GridPlugin,
            objects::ObjectsPlugin,
            scene::LoadableScenePlugin,
            selector::SelectorPlugin,
            waypoint::WaypointPlugin,
            raycast::RaycastPlugin,
            camera::CameraPlugin,
            physics::PhysicsPlugin,
            cursor::CursorPlugin,
            HanabiPlugin,
        ))
        .add_systems(Startup, startup)
        .add_systems(
            FixedUpdate,
            (window::resize_window.in_set(SystemStage::Spawn),),
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
