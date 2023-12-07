use bevy::{ecs::schedule::SystemSetConfigs, prelude::*};
use bevy_editor_pls::prelude::*;

mod aabb;
mod camera;
mod grid;
mod objects;
mod physics;
mod scene;
mod selector;
mod waypoint;
mod window;
mod zindex;

pub use aabb::Aabb2;

/// Stage of computation
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum SystemStage {
    Spawn,
    PreCompute,
    Compute,
    Apply,
    Despawn,
}
impl SystemStage {
    pub fn get_config() -> SystemSetConfigs {
        (Self::Compute, Self::Apply).chain()
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
            objects::ObjectsPlugin,
            scene::LoadableScenePlugin,
            selector::SelectorPlugin,
            camera::CameraPlugin,
            waypoint::WaypointPlugin,
        ))
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands) {
    commands.spawn((
        TextBundle::from_section(
            vec![
                "Controls:",
                "  Move camera: move mouse to border",
                "  Move waypoint: right click",
                "  Spawn zooids: 'z'",
                "  Despawn zooids: 'd'",
                "  Save scene: 's'",
                "  Open editor: 'e'",
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
