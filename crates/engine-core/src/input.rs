// Input system with rebindable action mappings

use winit::event::{ElementState, MouseButton};
use winit::keyboard::KeyCode;
use std::collections::{HashMap, HashSet};
use glam::Vec2;

/// Input actions that can be bound to keys/buttons
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputAction {
    // Movement
    MoveForward,
    MoveBackward,
    MoveLeft,
    MoveRight,
    Jump,
    Crouch,
    Sprint,

    // Camera
    LookUp,
    LookDown,
    LookLeft,
    LookRight,

    // Interaction
    Interact,
    Use,
    Reload,

    // Menu
    Pause,
    Inventory,
    Map,

    // Custom actions (for game-specific bindings)
    Custom(u32),
}

/// Input binding types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputBinding {
    Key(KeyCode),
    Mouse(MouseButton),
    MouseAxis(MouseAxis),
    GamepadButton(u32),
    GamepadAxis(u32),
}

/// Mouse axis for camera control
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseAxis {
    X,
    Y,
}

/// Input state manager with rebindable controls
pub struct InputState {
    // Current frame state
    keys_pressed: HashSet<KeyCode>,
    keys_just_pressed: HashSet<KeyCode>,
    keys_just_released: HashSet<KeyCode>,
    mouse_buttons_pressed: HashSet<MouseButton>,
    mouse_buttons_just_pressed: HashSet<MouseButton>,
    mouse_buttons_just_released: HashSet<MouseButton>,

    // Mouse state
    mouse_delta: Vec2,
    mouse_position: Vec2,
    scroll_delta: f32,

    // Action bindings
    action_bindings: HashMap<InputAction, Vec<InputBinding>>,

    // Input buffering for combos (stores recent inputs)
    input_buffer: Vec<(InputAction, f32)>,
    buffer_max_age: f32,
}

impl InputState {
    pub fn new() -> Self {
        let mut state = Self {
            keys_pressed: HashSet::new(),
            keys_just_pressed: HashSet::new(),
            keys_just_released: HashSet::new(),
            mouse_buttons_pressed: HashSet::new(),
            mouse_buttons_just_pressed: HashSet::new(),
            mouse_buttons_just_released: HashSet::new(),
            mouse_delta: Vec2::ZERO,
            mouse_position: Vec2::ZERO,
            scroll_delta: 0.0,
            action_bindings: HashMap::new(),
            input_buffer: Vec::new(),
            buffer_max_age: 0.5, // 500ms buffer
        };

        // Set up default bindings
        state.setup_default_bindings();
        state
    }

    /// Set up default WASD + mouse bindings
    fn setup_default_bindings(&mut self) {
        self.bind_action(InputAction::MoveForward, InputBinding::Key(KeyCode::KeyW));
        self.bind_action(InputAction::MoveBackward, InputBinding::Key(KeyCode::KeyS));
        self.bind_action(InputAction::MoveLeft, InputBinding::Key(KeyCode::KeyA));
        self.bind_action(InputAction::MoveRight, InputBinding::Key(KeyCode::KeyD));
        self.bind_action(InputAction::Jump, InputBinding::Key(KeyCode::Space));
        self.bind_action(InputAction::Crouch, InputBinding::Key(KeyCode::ControlLeft));
        self.bind_action(InputAction::Sprint, InputBinding::Key(KeyCode::ShiftLeft));

        self.bind_action(InputAction::Interact, InputBinding::Key(KeyCode::KeyE));
        self.bind_action(InputAction::Use, InputBinding::Mouse(MouseButton::Left));
        self.bind_action(InputAction::Reload, InputBinding::Key(KeyCode::KeyR));

        self.bind_action(InputAction::Pause, InputBinding::Key(KeyCode::Escape));
        self.bind_action(InputAction::Inventory, InputBinding::Key(KeyCode::Tab));
        self.bind_action(InputAction::Map, InputBinding::Key(KeyCode::KeyM));

        self.bind_action(InputAction::LookUp, InputBinding::MouseAxis(MouseAxis::Y));
        self.bind_action(InputAction::LookRight, InputBinding::MouseAxis(MouseAxis::X));
    }

    /// Update input state at start of frame
    pub fn update(&mut self, delta_time: f32) {
        // Clear "just pressed/released" states
        self.keys_just_pressed.clear();
        self.keys_just_released.clear();
        self.mouse_buttons_just_pressed.clear();
        self.mouse_buttons_just_released.clear();

        // Reset mouse delta (accumulates during event processing)
        self.mouse_delta = Vec2::ZERO;
        self.scroll_delta = 0.0;

        // Age out old inputs from buffer
        self.input_buffer.retain_mut(|(_, age)| {
            *age += delta_time;
            *age < self.buffer_max_age
        });
    }

    /// Handle keyboard input
    pub fn handle_keyboard_input(&mut self, keycode: KeyCode, state: ElementState) {
        match state {
            ElementState::Pressed => {
                if self.keys_pressed.insert(keycode) {
                    self.keys_just_pressed.insert(keycode);

                    // Add to input buffer
                    if let Some(action) = self.get_action_for_key(keycode) {
                        self.input_buffer.push((action, 0.0));
                    }
                }
            }
            ElementState::Released => {
                if self.keys_pressed.remove(&keycode) {
                    self.keys_just_released.insert(keycode);
                }
            }
        }
    }

    /// Handle mouse button input
    pub fn handle_mouse_button(&mut self, button: MouseButton, state: ElementState) {
        match state {
            ElementState::Pressed => {
                if self.mouse_buttons_pressed.insert(button) {
                    self.mouse_buttons_just_pressed.insert(button);

                    // Add to input buffer
                    if let Some(action) = self.get_action_for_mouse(button) {
                        self.input_buffer.push((action, 0.0));
                    }
                }
            }
            ElementState::Released => {
                if self.mouse_buttons_pressed.remove(&button) {
                    self.mouse_buttons_just_released.insert(button);
                }
            }
        }
    }

    /// Handle mouse movement
    pub fn handle_mouse_motion(&mut self, delta_x: f64, delta_y: f64) {
        self.mouse_delta.x += delta_x as f32;
        self.mouse_delta.y += delta_y as f32;
    }

    /// Handle mouse position
    pub fn handle_mouse_position(&mut self, x: f64, y: f64) {
        self.mouse_position = Vec2::new(x as f32, y as f32);
    }

    /// Handle scroll wheel
    pub fn handle_scroll(&mut self, delta: f32) {
        self.scroll_delta += delta;
    }

    /// Bind an action to an input
    pub fn bind_action(&mut self, action: InputAction, binding: InputBinding) {
        self.action_bindings
            .entry(action)
            .or_insert_with(Vec::new)
            .push(binding);
    }

    /// Unbind all inputs for an action
    pub fn unbind_action(&mut self, action: InputAction) {
        self.action_bindings.remove(&action);
    }

    /// Check if an action is currently active
    pub fn is_action_active(&self, action: InputAction) -> bool {
        if let Some(bindings) = self.action_bindings.get(&action) {
            for binding in bindings {
                match binding {
                    InputBinding::Key(key) => {
                        if self.keys_pressed.contains(key) {
                            return true;
                        }
                    }
                    InputBinding::Mouse(button) => {
                        if self.mouse_buttons_pressed.contains(button) {
                            return true;
                        }
                    }
                    InputBinding::MouseAxis(_) => {
                        // Axis inputs are handled by get_action_axis
                        continue;
                    }
                    _ => {} // Gamepad not implemented yet
                }
            }
        }
        false
    }

    /// Check if an action was just pressed this frame
    pub fn is_action_just_pressed(&self, action: InputAction) -> bool {
        if let Some(bindings) = self.action_bindings.get(&action) {
            for binding in bindings {
                match binding {
                    InputBinding::Key(key) => {
                        if self.keys_just_pressed.contains(key) {
                            return true;
                        }
                    }
                    InputBinding::Mouse(button) => {
                        if self.mouse_buttons_just_pressed.contains(button) {
                            return true;
                        }
                    }
                    _ => {}
                }
            }
        }
        false
    }

    /// Check if an action was just released this frame
    pub fn is_action_just_released(&self, action: InputAction) -> bool {
        if let Some(bindings) = self.action_bindings.get(&action) {
            for binding in bindings {
                match binding {
                    InputBinding::Key(key) => {
                        if self.keys_just_released.contains(key) {
                            return true;
                        }
                    }
                    InputBinding::Mouse(button) => {
                        if self.mouse_buttons_just_released.contains(button) {
                            return true;
                        }
                    }
                    _ => {}
                }
            }
        }
        false
    }

    /// Get axis value for an action (for mouse/gamepad axes)
    pub fn get_action_axis(&self, action: InputAction) -> f32 {
        if let Some(bindings) = self.action_bindings.get(&action) {
            for binding in bindings {
                if let InputBinding::MouseAxis(axis) = binding {
                    return match axis {
                        MouseAxis::X => self.mouse_delta.x,
                        MouseAxis::Y => self.mouse_delta.y,
                    };
                }
            }
        }
        0.0
    }

    /// Get movement vector from WASD-like bindings (normalized for diagonal movement)
    pub fn get_movement_vector(&self) -> Vec2 {
        let mut movement = Vec2::ZERO;

        if self.is_action_active(InputAction::MoveForward) {
            movement.y += 1.0;
        }
        if self.is_action_active(InputAction::MoveBackward) {
            movement.y -= 1.0;
        }
        if self.is_action_active(InputAction::MoveRight) {
            movement.x += 1.0;
        }
        if self.is_action_active(InputAction::MoveLeft) {
            movement.x -= 1.0;
        }

        // Normalize to prevent faster diagonal movement
        if movement.length_squared() > 0.0 {
            movement = movement.normalize();
        }

        movement
    }

    /// Get camera look delta
    pub fn get_look_delta(&self) -> Vec2 {
        Vec2::new(
            self.get_action_axis(InputAction::LookRight),
            self.get_action_axis(InputAction::LookUp),
        )
    }

    /// Check if a key is currently pressed
    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.keys_pressed.contains(&key)
    }

    /// Check if a key was just pressed
    pub fn is_key_just_pressed(&self, key: KeyCode) -> bool {
        self.keys_just_pressed.contains(&key)
    }

    /// Get mouse position
    pub fn mouse_position(&self) -> Vec2 {
        self.mouse_position
    }

    /// Get mouse delta (movement since last frame)
    pub fn mouse_delta(&self) -> Vec2 {
        self.mouse_delta
    }

    /// Get scroll delta
    pub fn scroll_delta(&self) -> f32 {
        self.scroll_delta
    }

    /// Get recent input buffer (for combo detection)
    pub fn get_input_buffer(&self) -> &[(InputAction, f32)] {
        &self.input_buffer
    }

    // Helper functions
    fn get_action_for_key(&self, key: KeyCode) -> Option<InputAction> {
        for (action, bindings) in &self.action_bindings {
            for binding in bindings {
                if let InputBinding::Key(k) = binding {
                    if k == &key {
                        return Some(*action);
                    }
                }
            }
        }
        None
    }

    fn get_action_for_mouse(&self, button: MouseButton) -> Option<InputAction> {
        for (action, bindings) in &self.action_bindings {
            for binding in bindings {
                if let InputBinding::Mouse(b) = binding {
                    if b == &button {
                        return Some(*action);
                    }
                }
            }
        }
        None
    }
}

impl Default for InputState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyboard_input() {
        let mut input = InputState::new();

        // Simulate key press
        input.handle_keyboard_input(KeyCode::KeyW, ElementState::Pressed);
        assert!(input.is_key_pressed(KeyCode::KeyW));
        assert!(input.is_key_just_pressed(KeyCode::KeyW));

        // After update, just_pressed should clear
        input.update(0.016);
        assert!(input.is_key_pressed(KeyCode::KeyW));
        assert!(!input.is_key_just_pressed(KeyCode::KeyW));

        // Release
        input.handle_keyboard_input(KeyCode::KeyW, ElementState::Released);
        assert!(!input.is_key_pressed(KeyCode::KeyW));
        assert!(input.is_key_just_released(KeyCode::KeyW));
    }

    #[test]
    fn test_action_binding() {
        let mut input = InputState::new();

        // W key should be bound to MoveForward by default
        input.handle_keyboard_input(KeyCode::KeyW, ElementState::Pressed);
        assert!(input.is_action_active(InputAction::MoveForward));
        assert!(input.is_action_just_pressed(InputAction::MoveForward));
    }

    #[test]
    fn test_movement_vector() {
        let mut input = InputState::new();

        // Forward only
        input.handle_keyboard_input(KeyCode::KeyW, ElementState::Pressed);
        let movement = input.get_movement_vector();
        assert_eq!(movement, Vec2::new(0.0, 1.0));

        // Diagonal (should be normalized)
        input.handle_keyboard_input(KeyCode::KeyD, ElementState::Pressed);
        let movement = input.get_movement_vector();
        assert!((movement.length() - 1.0).abs() < 0.001); // Should be normalized to length 1
    }

    #[test]
    fn test_mouse_delta() {
        let mut input = InputState::new();

        input.handle_mouse_motion(10.0, -5.0);
        assert_eq!(input.mouse_delta(), Vec2::new(10.0, -5.0));

        // Delta should reset after update
        input.update(0.016);
        assert_eq!(input.mouse_delta(), Vec2::ZERO);
    }

    #[test]
    fn test_input_buffer() {
        let mut input = InputState::new();

        input.handle_keyboard_input(KeyCode::Space, ElementState::Pressed);
        assert_eq!(input.get_input_buffer().len(), 1);
        assert_eq!(input.get_input_buffer()[0].0, InputAction::Jump);

        // Buffer should age out
        input.update(0.6); // More than buffer_max_age (0.5s)
        assert_eq!(input.get_input_buffer().len(), 0);
    }

    #[test]
    fn test_custom_binding() {
        let mut input = InputState::new();

        // Bind a custom action
        input.bind_action(InputAction::Custom(0), InputBinding::Key(KeyCode::KeyF));

        input.handle_keyboard_input(KeyCode::KeyF, ElementState::Pressed);
        assert!(input.is_action_active(InputAction::Custom(0)));
    }
}
