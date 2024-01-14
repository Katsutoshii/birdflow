/// Inputs are configured via an input map (TODO).
/// Mouse events are translated into InputActions.
/// Rays are cast to determine the target of the InputAction.
/// How can we determine what the target was?
use std::{
    ops::{Index, IndexMut},
    time::Duration,
};

use bevy::{prelude::*, sprite::Mesh2dHandle, utils::HashMap, window::PrimaryWindow};
use bevy_mod_raycast::primitives::Ray3d;

use crate::{prelude::*, raycast::raycast};

/// Plugin for input action events.
pub struct InputActionPlugin;
impl Plugin for InputActionPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<KeyCode>()
            .register_type::<MouseButton>()
            .add_event::<ControlEvent>()
            .add_event::<InputEvent>()
            .add_systems(
                FixedUpdate,
                (
                    InputEvent::update.in_set(SystemStage::Spawn),
                    ControlEvent::update.after(InputEvent::update),
                ),
            );
    }
}

/// Represents the state of an input.
#[derive(Event, Default, PartialEq, Clone, Copy, Debug, Hash)]
pub enum InputState {
    #[default]
    None,
    Start,
    Held,
    End,
}
/// Describes an action input by the user.
#[derive(Default, PartialEq, Clone, Copy, Debug, Hash)]
pub enum InputAction {
    #[default]
    None,
    Primary,
    Secondary,
    CameraPan,
}
impl From<InputAction> for MouseButton {
    fn from(value: InputAction) -> Self {
        match value {
            InputAction::Primary => MouseButton::Left,
            InputAction::Secondary => MouseButton::Right,
            InputAction::CameraPan => MouseButton::Middle,
            _ => MouseButton::Other(0),
        }
    }
}
#[derive(Event, Default, PartialEq, Clone, Copy, Debug)]
pub struct InputEvent {
    pub action: InputAction,
    pub state: InputState,
    pub ray: Ray3d,
}
impl InputEvent {
    fn process_input(input: &Input<MouseButton>, action: InputAction, ray: Ray3d) -> Option<Self> {
        let mouse_button = MouseButton::from(action);
        let state = if input.pressed(mouse_button) {
            if input.just_pressed(mouse_button) {
                InputState::Start
            } else {
                InputState::Held
            }
        } else if input.just_released(mouse_button) {
            InputState::End
        } else {
            InputState::None
        };
        if state != InputState::None {
            Some(Self { action, state, ray })
        } else {
            None
        }
    }
    pub fn update(
        camera: Query<(Entity, &Camera, &GlobalTransform), With<MainCamera>>,
        window: Query<&Window, With<PrimaryWindow>>,
        mouse_input: Res<Input<MouseButton>>,
        mut event_writer: EventWriter<Self>,
    ) {
        let mouse_buttons = [
            MouseButton::from(InputAction::Primary),
            MouseButton::from(InputAction::Secondary),
        ];
        if !mouse_input.any_pressed(mouse_buttons) && !mouse_input.any_just_released(mouse_buttons)
        {
            return;
        }

        let (_camera_entity, camera, camera_transform) = camera.single();
        let window = window.single();
        if let Some(position) = window.cursor_position() {
            let ray = Ray3d::from_screenspace(position, camera, camera_transform, window).unwrap();
            if let Some(event) = Self::process_input(&mouse_input, InputAction::Primary, ray) {
                event_writer.send(event);
            }
            if let Some(event) = Self::process_input(&mouse_input, InputAction::Secondary, ray) {
                event_writer.send(event);
            }
        }
    }
}

/// Describes an input action and the worldspace position where it occurred.
#[derive(Event, Default, Debug)]
pub struct ControlEvent {
    pub action: ControlAction,
    pub state: InputState,
    pub position: Vec2,
}
impl ControlEvent {
    fn get_control(
        event: &InputEvent,
        raycast_event: &RaycastEvent,
        grid_spec: &GridSpec,
    ) -> Option<Self> {
        match (raycast_event.target, event.action) {
            (RaycastTarget::None, _) => None,
            (RaycastTarget::WorldGrid, InputAction::Primary) => Some(Self {
                action: ControlAction::Select,
                state: event.state,
                position: raycast_event.world_position,
            }),
            (RaycastTarget::WorldGrid, InputAction::Secondary) => Some(Self {
                action: ControlAction::Move,
                state: event.state,
                position: raycast_event.world_position,
            }),
            (RaycastTarget::Minimap, InputAction::Primary) => Some(Self {
                action: ControlAction::PanCamera,
                state: event.state,
                position: grid_spec
                    .local_to_world_position(raycast_event.position * Vec2 { x: 1., y: -1. }),
            }),
            _ => None,
        }
    }
    pub fn update(
        meshes: Query<(Entity, &RaycastTarget, &Mesh2dHandle, &GlobalTransform)>,
        mesh_assets: Res<Assets<Mesh>>,
        mut input_events: EventReader<InputEvent>,
        mut event_writer: EventWriter<Self>,
        grid_spec: Res<GridSpec>,
        mut timers: Local<ControlTimers>,
        time: Res<Time>,
    ) {
        for event in input_events.read() {
            if let Some(raycast_event) = raycast(event.ray, meshes.iter(), &mesh_assets) {
                if let Some(control_event) = Self::get_control(event, &raycast_event, &grid_spec) {
                    if control_event.action == ControlAction::Move {
                        match control_event.state {
                            InputState::None => {}
                            InputState::Start => {
                                timers[ControlAction::Move].tick(time.delta());
                            }
                            InputState::Held => {
                                timers[ControlAction::Move].tick(time.delta());
                                if !timers[ControlAction::Move].finished() {
                                    continue;
                                }
                            }
                            InputState::End => {
                                timers[ControlAction::Move].reset();
                            }
                        }
                    }
                    event_writer.send(control_event);
                }
            }
        }
    }
}

/// Describes an action input by the user.
#[derive(Default, PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub enum ControlAction {
    #[default]
    None,
    Select,
    Move,
    PanCamera,
}

/// Collection of timers to prevent input action spam.
#[derive(Deref, DerefMut)]
pub struct ControlTimers(HashMap<ControlAction, Timer>);
impl Default for ControlTimers {
    fn default() -> Self {
        let mut timers = Self(HashMap::default());
        timers.insert(
            ControlAction::Move,
            Timer::new(Duration::from_millis(100), TimerMode::Repeating),
        );
        timers
    }
}
impl Index<ControlAction> for ControlTimers {
    type Output = Timer;
    fn index(&self, i: ControlAction) -> &Self::Output {
        self.get(&i).unwrap()
    }
}
impl IndexMut<ControlAction> for ControlTimers {
    fn index_mut(&mut self, i: ControlAction) -> &mut Self::Output {
        self.get_mut(&i).unwrap()
    }
}
