use bevy::prelude::*;
use bevy_editor_pls::prelude::*;

mod bird;
mod scene;

/// Used to help identify our main camera
#[derive(Component)]
pub struct MainCamera;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(AssetPlugin {
                watch_for_changes_override: Some(true),
                ..default()
            }),
            EditorPlugin::default(),
            bird::BirdsPlugin,
            scene::LoadableScenePlugin,
        ))
        .register_type::<Name>()
        .add_systems(Startup, startup)
        .run();
}

fn startup(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), MainCamera));
    commands.spawn(
        TextBundle::from_section(
            vec![
                "Controls:",
                "Spawn birds: Left click",
                "Despawn birds: 'd'",
                "Save scene: 's'",
                "Open editor: 'e'",
            ]
            .join("\n"),
            TextStyle {
                font_size: 20.0,
                ..default()
            },
        )
        .with_style(Style {
            align_self: AlignSelf::FlexEnd,
            ..default()
        }),
    );
}
