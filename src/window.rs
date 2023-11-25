use bevy::{
    prelude::*,
    window::{Cursor, CursorGrabMode, PresentMode, WindowTheme},
};

pub const DIMENSIONS: Vec2 = Vec2 { x: 1600., y: 900. };

pub fn custom_plugin() -> WindowPlugin {
    WindowPlugin {
        primary_window: Some(Window {
            cursor: Cursor {
                grab_mode: CursorGrabMode::Confined,
                ..default()
            },
            title: "Bevy Birds".into(),
            resolution: DIMENSIONS.into(),
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
            ..default()
        }),
        ..default()
    }
}
