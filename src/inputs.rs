use std::{
    ops::{Index, IndexMut},
    time::Duration,
};

use bevy::{prelude::*, utils::HashMap, window::PrimaryWindow};

use crate::prelude::*;

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

/// Describes an input action and the worldspace position where it occurred.
#[derive(Event, Default)]
pub struct InputActionEvent {
    pub action: InputAction,
    pub position: Vec2,
}

/// Collection of timers to prevent input action spam.
#[derive(Deref, DerefMut)]
pub struct InputTimers(HashMap<InputAction, Timer>);
impl Default for InputTimers {
    fn default() -> Self {
        let mut timers = Self(HashMap::default());
        timers.insert(
            InputAction::Move,
            Timer::new(Duration::from_millis(100), TimerMode::Repeating),
        );
        timers
    }
}
impl Index<InputAction> for InputTimers {
    type Output = Timer;
    fn index(&self, i: InputAction) -> &Self::Output {
        self.get(&i).unwrap()
    }
}
impl IndexMut<InputAction> for InputTimers {
    fn index_mut(&mut self, i: InputAction) -> &mut Self::Output {
        self.get_mut(&i).unwrap()
    }
}

/// Describes an action input by the user.
#[derive(Default, PartialEq, Eq, Clone, Copy, Hash)]
pub enum InputAction {
    #[default]
    None,
    StartSelect,
    Select,
    EndSelect,
    StartMove,
    Move,
}
impl InputAction {
    pub fn update(
        camera_query: Query<(Entity, &Camera, &GlobalTransform), With<MainCamera>>,
        window_query: Query<&Window, With<PrimaryWindow>>,
        mouse_input: Res<Input<MouseButton>>,
        mut event_writer: EventWriter<InputActionEvent>,
        mut timers: Local<InputTimers>,
        time: Res<Time>,
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
        if let Some(ray) = window_query
            .single()
            .cursor_position()
            .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        {
            let position = ray.origin.xy();
            // Movement
            if mouse_input.just_pressed(MouseButton::Right) {
                event_writer.send(InputActionEvent {
                    action: InputAction::StartMove,
                    position,
                })
            } else if mouse_input.pressed(MouseButton::Right) {
                timers[InputAction::Move].tick(time.delta());
                if timers[InputAction::Move].finished() {
                    event_writer.send(InputActionEvent {
                        action: InputAction::Move,
                        position,
                    })
                }
            } else if mouse_input.just_released(MouseButton::Right) {
                timers[InputAction::Move].reset();
                event_writer.send(InputActionEvent {
                    action: InputAction::Move,
                    position,
                })
            }

            // Selection
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
                event_writer.send(InputActionEvent {
                    action: InputAction::EndSelect,
                    position,
                })
            }
        }
    }
}
