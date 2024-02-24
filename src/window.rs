use bevy::{
    prelude::*,
    window::{Cursor, PresentMode, PrimaryWindow, WindowMode, WindowTheme},
};

use crate::prelude::Configs;

pub fn custom_plugin() -> WindowPlugin {
    WindowPlugin {
        primary_window: Some(Window {
            cursor: Cursor {
                visible: false,
                ..default()
            },
            title: "Bevy Zooids".into(),
            present_mode: PresentMode::AutoVsync,
            // Tells wasm to resize the window according to the available canvas
            fit_canvas_to_parent: true,
            // Tells wasm not to override default event handling, like F5, Ctrl+R etc.
            prevent_default_event_handling: false,
            window_theme: Some(WindowTheme::Dark),
            enabled_buttons: bevy::window::EnabledButtons {
                maximize: false,
                ..Default::default()
            },
            visible: true,
            resizable: false,
            mode: WindowMode::BorderlessFullscreen,
            ..default()
        }),
        ..default()
    }
}

pub fn resize_window(mut query: Query<&mut Window, With<PrimaryWindow>>, configs: Res<Configs>) {
    if configs.is_changed() {
        let mut window = query.single_mut();
        let scale_factor = window.scale_factor() as f32;
        if configs.window_size != Vec2::ZERO {
            window.resolution.set_physical_resolution(
                (configs.window_size.x * scale_factor) as u32,
                (configs.window_size.y * scale_factor) as u32,
            );
        }
    }
}
