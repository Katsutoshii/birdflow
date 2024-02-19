/// Inputs are configured via an input map (TODO).
/// Mouse events are translated into InputActions.
/// Rays are cast to determine the target of the InputAction.
/// How can we determine what the target was?
use std::{
    ops::{Index, IndexMut},
    time::Duration,
};

use bevy::{prelude::*, sprite::Mesh2dHandle, utils::HashMap};
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
                Update,
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
    Pressed,
    Held,
    Released,
}

pub enum RawInput {
    MouseButton(MouseButton),
    KeyCode(KeyCode),
}

/// Describes an action input by the user.
#[derive(Default, PartialEq, Clone, Copy, Debug, Hash)]
pub enum InputAction {
    #[default]
    None,
    Primary,
    Secondary,
    CameraPan,
    SpawnHead,
    SpawnZooid,
    SpawnRed,
    SpawnBlue,
}
impl InputAction {
    const NUM_ACTIONS: usize = 7;
    const ACTIONS: [Self; Self::NUM_ACTIONS] = [
        Self::Primary,
        Self::Secondary,
        Self::CameraPan,
        Self::SpawnHead,
        Self::SpawnZooid,
        Self::SpawnRed,
        Self::SpawnBlue,
    ];
    pub fn mouse_buttons() -> Vec<MouseButton> {
        let mut result = Vec::new();
        for action in Self::ACTIONS {
            if let RawInput::MouseButton(mouse_button) = RawInput::from(action) {
                result.push(mouse_button);
            }
        }
        result
    }
    pub fn key_codes() -> Vec<KeyCode> {
        let mut result = Vec::new();
        for action in Self::ACTIONS {
            if let RawInput::KeyCode(key_code) = RawInput::from(action) {
                result.push(key_code);
            }
        }
        result
    }
}
impl From<InputAction> for RawInput {
    fn from(value: InputAction) -> Self {
        match value {
            InputAction::None => unreachable!(),
            InputAction::Primary => Self::MouseButton(MouseButton::Left),
            InputAction::Secondary => Self::MouseButton(MouseButton::Right),
            InputAction::CameraPan => Self::MouseButton(MouseButton::Middle),
            InputAction::SpawnHead => Self::KeyCode(KeyCode::Return),
            InputAction::SpawnRed => Self::KeyCode(KeyCode::Minus),
            InputAction::SpawnBlue => Self::KeyCode(KeyCode::Equals),
            InputAction::SpawnZooid => Self::KeyCode(KeyCode::Z),
        }
    }
}
impl InputAction {}

#[derive(Event, Default, PartialEq, Clone, Copy, Debug)]
pub struct InputEvent {
    pub action: InputAction,
    pub state: InputState,
    pub ray: Ray3d,
}
impl InputEvent {
    fn process_input(
        input: &ButtonInput<MouseButton>,
        keyboard_input: &ButtonInput<KeyCode>,
        action: InputAction,
        ray: Ray3d,
    ) -> Option<Self> {
        match RawInput::from(action) {
            RawInput::MouseButton(mouse_button) => {
                let state = if input.pressed(mouse_button) {
                    if input.just_pressed(mouse_button) {
                        InputState::Pressed
                    } else {
                        InputState::Held
                    }
                } else if input.just_released(mouse_button) {
                    InputState::Released
                } else {
                    InputState::None
                };
                if state != InputState::None {
                    Some(Self { action, state, ray })
                } else {
                    None
                }
            }
            RawInput::KeyCode(key_code) => {
                let state = if keyboard_input.pressed(key_code) {
                    if keyboard_input.just_pressed(key_code) {
                        InputState::Pressed
                    } else {
                        InputState::Held
                    }
                } else if keyboard_input.just_released(key_code) {
                    InputState::Released
                } else {
                    InputState::None
                };
                if state != InputState::None {
                    Some(Self { action, state, ray })
                } else {
                    None
                }
            }
        }
    }

    pub fn update(
        mouse_input: Res<ButtonInput<MouseButton>>,
        keyboard_input: Res<ButtonInput<KeyCode>>,
        cursor: Query<&GlobalTransform, With<Cursor>>,
        mut event_writer: EventWriter<Self>,
    ) {
        let cursor = cursor.single();
        let ray = Ray3d::new(cursor.translation(), -Vec3::Z);
        for action in InputAction::ACTIONS {
            if let Some(event) = Self::process_input(&mouse_input, &keyboard_input, action, ray) {
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
    pub fn is_pressed(&self, action: ControlAction) -> bool {
        self.action == action && self.state == InputState::Pressed
    }
    pub fn is_held(&self, action: ControlAction) -> bool {
        self.action == action && self.state == InputState::Held
    }
    pub fn is_released(&self, action: ControlAction) -> bool {
        self.action == action && self.state == InputState::Released
    }
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
            (_, InputAction::SpawnHead) => {
                if event.state == InputState::Pressed {
                    info!("SpawnHead");
                }
                Some(Self {
                    action: ControlAction::SpawnHead,
                    state: event.state,
                    position: raycast_event.world_position,
                })
            }
            (_, InputAction::SpawnZooid) => Some(Self {
                action: ControlAction::SpawnZooid,
                state: event.state,
                position: raycast_event.world_position,
            }),
            (_, InputAction::SpawnRed) => Some(Self {
                action: ControlAction::SpawnRed,
                state: event.state,
                position: raycast_event.world_position,
            }),
            (_, InputAction::SpawnBlue) => Some(Self {
                action: ControlAction::SpawnBlue,
                state: event.state,
                position: raycast_event.world_position,
            }),
            _ => None,
        }
    }
    pub fn update(
        meshes: Query<(Entity, &RaycastTarget, &Mesh2dHandle, &GlobalTransform)>,
        mesh_assets: Res<Assets<Mesh>>,
        mut input_events: EventReader<InputEvent>,
        mut event_writer: EventWriter<Self>,
        grid_spec: Option<Res<GridSpec>>,
        mut timers: Local<ControlTimers>,
        time: Res<Time>,
    ) {
        let grid_spec = if let Some(grid_spec) = grid_spec {
            grid_spec
        } else {
            return;
        };
        for event in input_events.read() {
            if let Some(raycast_event) = raycast(event.ray, meshes.iter(), &mesh_assets) {
                if let Some(control_event) = Self::get_control(event, &raycast_event, &grid_spec) {
                    if control_event.action == ControlAction::Move {
                        match control_event.state {
                            InputState::None => {}
                            InputState::Pressed => {
                                timers[ControlAction::Move].reset();
                                timers[ControlAction::Move].tick(time.delta());
                            }
                            InputState::Held => {
                                timers[ControlAction::Move].tick(time.delta());
                                if !timers[ControlAction::Move].finished() {
                                    continue;
                                }
                            }
                            InputState::Released => {
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

    SpawnHead,
    SpawnZooid,
    SpawnRed,
    SpawnBlue,
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
