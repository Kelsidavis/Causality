// Engine Input - Comprehensive input system for keyboard, mouse, and gamepad

pub mod keyboard;
pub mod mouse;
pub mod gamepad;
pub mod actions;
pub mod manager;

pub use keyboard::{KeyCode, KeyboardState};
pub use mouse::{MouseButton, MouseState};
pub use gamepad::{GamepadAxis, GamepadButton, GamepadId, GamepadState};
pub use actions::{InputAction, InputActionMap, ActionBinding, BindingType};
pub use manager::InputManager;

/// Input axis value (-1.0 to 1.0)
pub type AxisValue = f32;

/// Common input axes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Axis {
    /// Horizontal movement (left/right)
    Horizontal,
    /// Vertical movement (forward/back)
    Vertical,
    /// Look horizontal (camera yaw)
    LookHorizontal,
    /// Look vertical (camera pitch)
    LookVertical,
    /// Custom axis
    Custom(&'static str),
}

/// Input events
#[derive(Debug, Clone)]
pub enum InputEvent {
    KeyPressed(KeyCode),
    KeyReleased(KeyCode),
    MouseButtonPressed(MouseButton),
    MouseButtonReleased(MouseButton),
    MouseMoved { delta_x: f32, delta_y: f32 },
    MouseScrolled { delta: f32 },
    GamepadButtonPressed(GamepadId, GamepadButton),
    GamepadButtonReleased(GamepadId, GamepadButton),
    GamepadAxisChanged(GamepadId, GamepadAxis, f32),
    GamepadConnected(GamepadId),
    GamepadDisconnected(GamepadId),
}
