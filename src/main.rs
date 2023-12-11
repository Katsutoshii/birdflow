use bevy::{ecs::schedule::SystemSetConfigs, prelude::*};
use bevy_editor_pls::prelude::*;

pub mod aabb;
pub mod camera;
pub mod fog;
pub mod grid;
pub mod objects;
pub mod physics;
pub mod scene;
pub mod selector;
pub mod window;
pub mod zindex;

pub mod prelude {
    pub use crate::{
        aabb::Aabb2,
        camera::MainCamera,
        grid,
        physics::{NewVelocity, Velocity},
        selector::Selected,
        zindex, SystemStage,
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
            EditorPlugin::default(),
            grid::GridPlugin,
            fog::FogPlugin,
            objects::ObjectsPlugin,
            scene::LoadableScenePlugin,
            selector::SelectorPlugin,
            camera::CameraPlugin,
            physics::PhysicsPlugin,
        ))
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands) {
    commands.spawn((
        TextBundle::from_section(
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
        }),
        scene::SaveEntity,
    ));
}
