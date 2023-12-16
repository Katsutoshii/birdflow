use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::grid::GridSpec;
use crate::{window, Aabb2};

pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::BLACK))
            .add_systems(Startup, MainCamera::startup)
            .add_systems(
                FixedUpdate,
                (
                    CameraController::update_bounds,
                    CameraController::update,
                    CameraController::update_drag,
                ),
            );
    }
}

/// Used to help identify our main camera
#[derive(Component)]
pub struct MainCamera;
impl MainCamera {
    pub fn startup(mut commands: Commands) {
        commands.spawn((
            Camera2dBundle::default(),
            CameraController::default(),
            MainCamera,
        ));
    }
}

#[derive(Component)]
pub struct CameraController {
    pub enabled: bool,
    pub initialized: bool,
    pub sensitivity: f32,
    pub velocity: Vec2,
    pub last_drag_position: Option<Vec2>,
    world2d_bounds: Aabb2,
}
impl Default for CameraController {
    fn default() -> Self {
        Self {
            enabled: true,
            initialized: false,
            sensitivity: 1000.0,
            velocity: Vec2::ZERO,
            last_drag_position: None,
            world2d_bounds: Aabb2::default(),
        }
    }
}

impl CameraController {
    fn update_bounds(
        grid_spec: Res<GridSpec>,
        mut controller_query: Query<(&mut Self, &Camera, &GlobalTransform), With<MainCamera>>,
    ) {
        if !grid_spec.is_changed() {
            return;
        }
        let (mut controller, camera, camera_transform) = controller_query.single_mut();
        if let Some(world2d_size) = Self::get_world2d_size(camera, camera_transform) {
            controller.world2d_bounds = grid_spec.world2d_bounds();
            controller.world2d_bounds.min += world2d_size * 0.5;
            controller.world2d_bounds.max -= world2d_size * 0.5;
        }
    }

    fn get_world2d_size(camera: &Camera, camera_transform: &GlobalTransform) -> Option<Vec2> {
        let camera_min = camera.viewport_to_world_2d(
            camera_transform,
            Vec2 {
                x: 0.,
                y: window::DIMENSIONS.y,
            },
        )?;
        let camera_max = camera.viewport_to_world_2d(
            camera_transform,
            Vec2 {
                x: window::DIMENSIONS.x,
                y: 0.,
            },
        )?;
        Some(camera_max - camera_min)
    }

    pub fn update_drag(
        mut controller_query: Query<
            (&mut Self, &mut Transform, &Camera, &GlobalTransform),
            With<MainCamera>,
        >,
        window_query: Query<&Window, With<PrimaryWindow>>,
        mouse_input: Res<Input<MouseButton>>,
    ) {
        let window = window_query.single();
        let (mut controller, mut camera_transform, camera, camera_global_transform) =
            controller_query.single_mut();

        if let Some(cursor_position) = window.cursor_position() {
            // Middle mouse drag
            if mouse_input.pressed(MouseButton::Middle) {
                if let Some(cursor_world2d) =
                    camera.viewport_to_world_2d(camera_global_transform, cursor_position)
                {
                    if let Some(last_drag_position) = controller.last_drag_position {
                        let delta = last_drag_position - cursor_world2d;
                        // Why is this 0.5 here?
                        // Must have something to do with the default orthographic projection...
                        // This is probably why it feels so slow, too.
                        camera_transform.translation += (0.5 * delta).extend(0.);
                    }
                    controller.last_drag_position = Some(cursor_world2d);
                }
            } else if mouse_input.just_released(MouseButton::Middle) {
                controller.last_drag_position = None;
            }
        }
        controller
            .world2d_bounds
            .clamp3(&mut camera_transform.translation)
    }

    pub fn update(
        time: Res<Time>,
        mut controller_query: Query<(&mut Self, &mut Transform), With<MainCamera>>,
        window_query: Query<&Window, With<PrimaryWindow>>,
    ) {
        let dt = time.delta_seconds();
        let window = window_query.single();
        let (mut controller, mut camera_transform) = controller_query.single_mut();

        let mut acceleration = Vec2::ZERO;
        controller.velocity = Vec2::ZERO;
        if let Some(cursor_position) = window.cursor_position() {
            // Screen border panning.
            acceleration += if cursor_position.x < 1. {
                -Vec2::X
            } else if cursor_position.x > window::DIMENSIONS.x - 1. {
                Vec2::X
            } else {
                Vec2::ZERO
            };
            acceleration += if cursor_position.y < 1. {
                Vec2::Y
            } else if cursor_position.y > window::DIMENSIONS.y - 1. {
                -Vec2::Y
            } else {
                Vec2::ZERO
            };
        }
        controller.velocity += acceleration;
        camera_transform.translation +=
            controller.velocity.extend(0.) * dt * controller.sensitivity;
        controller
            .world2d_bounds
            .clamp3(&mut camera_transform.translation)
    }
}
