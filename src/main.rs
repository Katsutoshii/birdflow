use bevy::prelude::*;
use bevy_editor_pls::prelude::*;

mod aabb;
mod bird;
mod camera;
mod grid;
mod scene;
mod waypoint;
mod window;
mod zindex;

pub use aabb::Aabb2;

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
            bird::BirdsPlugin,
            scene::LoadableScenePlugin,
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
                "  Spawn birds: 'b'",
                "  Despawn birds: 'd'",
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
