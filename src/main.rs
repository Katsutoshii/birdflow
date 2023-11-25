use bevy::{prelude::*, window::PrimaryWindow};
use bevy_editor_pls::prelude::*;
use grid::EntityGrid;

mod aabb;
mod bird;
mod camera;
mod grid;
mod scene;
mod window;

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
        ))
        .register_type::<Name>()
        .register_type::<core::num::NonZeroU16>()
        .add_systems(Startup, startup)
        .add_systems(FixedUpdate, position_debug)
        .run();
}

fn startup(mut commands: Commands) {
    commands.spawn((
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
        scene::SaveEntity,
    ));
}

fn position_debug(
    camera_query: Query<(Entity, &Camera, &GlobalTransform), With<camera::MainCamera>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mouse_input: Res<Input<MouseButton>>,
    _grid: ResMut<EntityGrid>,
) {
    if !mouse_input.just_pressed(MouseButton::Left) {
        return;
    }
    let (_camera_entity, camera, camera_transform) = camera_query.single();
    if let Some(position) = window_query
        .single()
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor))
    {
        info!("Clicked on position: {}", position);
        // grid.update(camera_entity, position);
    }
}
