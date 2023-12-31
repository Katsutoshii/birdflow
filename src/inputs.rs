use bevy::{prelude::*, window::PrimaryWindow};

use crate::{camera::MainCamera, SystemStage};

/// Plugin for input action events.
pub struct InputActionPlugin;
impl Plugin for InputActionPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<KeyCode>()
            .register_type::<MouseButton>()
            .add_event::<InputActionEvent>()
            .add_systems(FixedUpdate, InputAction::update.in_set(SystemStage::Spawn));
    }
}

// Describes an input action.
#[derive(Event, Default)]
pub struct InputActionEvent {
    pub action: InputAction,
    pub position: Vec2,
}
#[derive(Default, PartialEq, Eq, Clone, Copy)]
pub enum InputAction {
    #[default]
    None,
    StartSelect,
    Select,
    EndSelect,
    StartMove,
    Move,
    EndMove,
}

impl InputAction {
    pub fn update(
        camera_query: Query<(Entity, &Camera, &GlobalTransform), With<MainCamera>>,
        window_query: Query<&Window, With<PrimaryWindow>>,
        mouse_input: Res<Input<MouseButton>>,
        mut event_writer: EventWriter<InputActionEvent>,
    ) {
        if !mouse_input.any_pressed([MouseButton::Right, MouseButton::Left, MouseButton::Middle])
            && !mouse_input.any_just_released([
                MouseButton::Right,
                MouseButton::Left,
                MouseButton::Middle,
            ])
        {
            return;
        }
        let (_camera_entity, camera, camera_transform) = camera_query.single();
        if let Some(position) = window_query
            .single()
            .cursor_position()
            .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor))
        {
            if mouse_input.just_pressed(MouseButton::Right) {
                event_writer.send(InputActionEvent {
                    action: InputAction::StartMove,
                    position,
                })
            } else if mouse_input.pressed(MouseButton::Right) {
                event_writer.send(InputActionEvent {
                    action: InputAction::Move,
                    position,
                })
            } else if mouse_input.just_released(MouseButton::Right) {
                event_writer.send(InputActionEvent {
                    action: InputAction::EndMove,
                    position,
                })
            }
            if mouse_input.just_pressed(MouseButton::Left) {
                event_writer.send(InputActionEvent {
                    action: InputAction::StartSelect,
                    position,
                })
            } else if mouse_input.pressed(MouseButton::Left) {
                event_writer.send(InputActionEvent {
                    action: InputAction::Select,
                    position,
                })
            } else if mouse_input.just_released(MouseButton::Left) {
                info!("EndSelect!");
                event_writer.send(InputActionEvent {
                    action: InputAction::EndSelect,
                    position,
                })
            }
        }
    }
}
