// Gamepad/Controller input handling

use std::collections::HashMap;
use gilrs::{Gilrs, GamepadId as GilrsGamepadId, Axis as GilrsAxis, Button as GilrsButton, Event, EventType};

/// Gamepad identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GamepadId(pub usize);

impl From<GilrsGamepadId> for GamepadId {
    fn from(id: GilrsGamepadId) -> Self {
        GamepadId(id.into())
    }
}

// Removed - GilrsGamepadId::from() expects usize directly
// The conversion is already handled by the fact that GamepadId.0 is usize

/// Gamepad buttons
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum GamepadButton {
    // Face buttons
    South,       // A on Xbox, X on PlayStation
    East,        // B on Xbox, Circle on PlayStation
    West,        // X on Xbox, Square on PlayStation
    North,       // Y on Xbox, Triangle on PlayStation

    // D-pad
    DPadUp,
    DPadDown,
    DPadLeft,
    DPadRight,

    // Shoulder buttons
    LeftShoulder,
    RightShoulder,
    LeftTrigger2,   // L2
    RightTrigger2,  // R2

    // Stick buttons
    LeftThumb,
    RightThumb,

    // Menu buttons
    Start,
    Select,
    Mode,

    Unknown,
}

impl From<GilrsButton> for GamepadButton {
    fn from(button: GilrsButton) -> Self {
        match button {
            GilrsButton::South => GamepadButton::South,
            GilrsButton::East => GamepadButton::East,
            GilrsButton::North => GamepadButton::North,
            GilrsButton::West => GamepadButton::West,
            GilrsButton::DPadUp => GamepadButton::DPadUp,
            GilrsButton::DPadDown => GamepadButton::DPadDown,
            GilrsButton::DPadLeft => GamepadButton::DPadLeft,
            GilrsButton::DPadRight => GamepadButton::DPadRight,
            GilrsButton::LeftTrigger => GamepadButton::LeftShoulder,
            GilrsButton::RightTrigger => GamepadButton::RightShoulder,
            GilrsButton::LeftTrigger2 => GamepadButton::LeftTrigger2,
            GilrsButton::RightTrigger2 => GamepadButton::RightTrigger2,
            GilrsButton::LeftThumb => GamepadButton::LeftThumb,
            GilrsButton::RightThumb => GamepadButton::RightThumb,
            GilrsButton::Start => GamepadButton::Start,
            GilrsButton::Select => GamepadButton::Select,
            GilrsButton::Mode => GamepadButton::Mode,
            _ => GamepadButton::Unknown,
        }
    }
}

/// Gamepad axes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum GamepadAxis {
    LeftStickX,
    LeftStickY,
    RightStickX,
    RightStickY,
    LeftTrigger,
    RightTrigger,
    Unknown,
}

impl From<GilrsAxis> for GamepadAxis {
    fn from(axis: GilrsAxis) -> Self {
        match axis {
            GilrsAxis::LeftStickX => GamepadAxis::LeftStickX,
            GilrsAxis::LeftStickY => GamepadAxis::LeftStickY,
            GilrsAxis::RightStickX => GamepadAxis::RightStickX,
            GilrsAxis::RightStickY => GamepadAxis::RightStickY,
            GilrsAxis::LeftZ => GamepadAxis::LeftTrigger,
            GilrsAxis::RightZ => GamepadAxis::RightTrigger,
            _ => GamepadAxis::Unknown,
        }
    }
}

/// Single gamepad state
#[derive(Debug, Clone)]
pub struct GamepadState {
    pub id: GamepadId,
    pub name: String,
    pub connected: bool,
    buttons: HashMap<GamepadButton, bool>,
    axes: HashMap<GamepadAxis, f32>,
    /// Dead zone for analog sticks (0.0 to 1.0)
    pub dead_zone: f32,
}

impl GamepadState {
    pub fn new(id: GamepadId, name: String) -> Self {
        Self {
            id,
            name,
            connected: true,
            buttons: HashMap::new(),
            axes: HashMap::new(),
            dead_zone: 0.15, // Default dead zone
        }
    }

    /// Check if a button is pressed
    pub fn is_button_pressed(&self, button: GamepadButton) -> bool {
        self.buttons.get(&button).copied().unwrap_or(false)
    }

    /// Get axis value (-1.0 to 1.0)
    pub fn axis_value(&self, axis: GamepadAxis) -> f32 {
        let value = self.axes.get(&axis).copied().unwrap_or(0.0);

        // Apply dead zone
        if value.abs() < self.dead_zone {
            0.0
        } else {
            // Rescale to remove dead zone (smooth transition)
            let sign = value.signum();
            let magnitude = (value.abs() - self.dead_zone) / (1.0 - self.dead_zone);
            sign * magnitude
        }
    }

    /// Get left stick as 2D vector
    pub fn left_stick(&self) -> (f32, f32) {
        (
            self.axis_value(GamepadAxis::LeftStickX),
            self.axis_value(GamepadAxis::LeftStickY),
        )
    }

    /// Get right stick as 2D vector
    pub fn right_stick(&self) -> (f32, f32) {
        (
            self.axis_value(GamepadAxis::RightStickX),
            self.axis_value(GamepadAxis::RightStickY),
        )
    }

    /// Update button state
    pub(crate) fn set_button(&mut self, button: GamepadButton, pressed: bool) {
        self.buttons.insert(button, pressed);
    }

    /// Update axis value
    pub(crate) fn set_axis(&mut self, axis: GamepadAxis, value: f32) {
        self.axes.insert(axis, value);
    }
}

/// Gamepad manager
pub struct GamepadManager {
    gilrs: Gilrs,
    gamepads: HashMap<GamepadId, GamepadState>,
}

impl GamepadManager {
    pub fn new() -> Result<Self, String> {
        let gilrs = Gilrs::new().map_err(|e| format!("Failed to initialize gamepad support: {}", e))?;
        let mut gamepads = HashMap::new();

        // Detect already connected gamepads
        for (id, gamepad) in gilrs.gamepads() {
            let gamepad_id = GamepadId::from(id);
            let name = gamepad.name().to_string();
            log::info!("Gamepad connected: {} (ID: {:?})", name, gamepad_id);
            gamepads.insert(gamepad_id, GamepadState::new(gamepad_id, name));
        }

        Ok(Self { gilrs, gamepads })
    }

    /// Update gamepad states (call once per frame)
    pub fn update(&mut self) -> Vec<crate::InputEvent> {
        let mut events = Vec::new();

        while let Some(Event { id, event, .. }) = self.gilrs.next_event() {
            let gamepad_id = GamepadId::from(id);

            match event {
                EventType::ButtonPressed(button, _) => {
                    let btn = GamepadButton::from(button);
                    if let Some(gamepad) = self.gamepads.get_mut(&gamepad_id) {
                        gamepad.set_button(btn, true);
                    }
                    events.push(crate::InputEvent::GamepadButtonPressed(gamepad_id, btn));
                }
                EventType::ButtonReleased(button, _) => {
                    let btn = GamepadButton::from(button);
                    if let Some(gamepad) = self.gamepads.get_mut(&gamepad_id) {
                        gamepad.set_button(btn, false);
                    }
                    events.push(crate::InputEvent::GamepadButtonReleased(gamepad_id, btn));
                }
                EventType::AxisChanged(axis, value, _) => {
                    let ax = GamepadAxis::from(axis);
                    if let Some(gamepad) = self.gamepads.get_mut(&gamepad_id) {
                        gamepad.set_axis(ax, value);
                    }
                    events.push(crate::InputEvent::GamepadAxisChanged(gamepad_id, ax, value));
                }
                EventType::Connected => {
                    let gamepad = self.gilrs.gamepad(id);
                    let name = gamepad.name().to_string();
                    log::info!("Gamepad connected: {} (ID: {:?})", name, gamepad_id);
                    self.gamepads.insert(gamepad_id, GamepadState::new(gamepad_id, name));
                    events.push(crate::InputEvent::GamepadConnected(gamepad_id));
                }
                EventType::Disconnected => {
                    log::info!("Gamepad disconnected: ID {:?}", gamepad_id);
                    if let Some(gamepad) = self.gamepads.get_mut(&gamepad_id) {
                        gamepad.connected = false;
                    }
                    events.push(crate::InputEvent::GamepadDisconnected(gamepad_id));
                }
                _ => {}
            }
        }

        events
    }

    /// Get gamepad by ID
    pub fn gamepad(&self, id: GamepadId) -> Option<&GamepadState> {
        self.gamepads.get(&id)
    }

    /// Get all connected gamepads
    pub fn gamepads(&self) -> impl Iterator<Item = &GamepadState> {
        self.gamepads.values().filter(|g| g.connected)
    }

    /// Get first connected gamepad (convenience method)
    pub fn first_gamepad(&self) -> Option<&GamepadState> {
        self.gamepads()
            .next()
    }
}

impl Default for GamepadManager {
    fn default() -> Self {
        Self::new().unwrap_or_else(|e| {
            log::warn!("Failed to initialize gamepad support: {}", e);
            Self {
                gilrs: Gilrs::new().unwrap(),
                gamepads: HashMap::new(),
            }
        })
    }
}
