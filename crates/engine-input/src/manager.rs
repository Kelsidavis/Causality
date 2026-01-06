// Input manager - Main API for input system

use super::{
    KeyboardState, MouseState, GamepadId,
    InputActionMap, InputAction, BindingType, InputEvent, AxisValue,
    KeyCode, MouseButton, GamepadButton, GamepadAxis,
};
use super::gamepad::GamepadManager;
use winit::event::{ElementState, WindowEvent, MouseScrollDelta, DeviceEvent};
use glam::Vec2;

/// Main input manager
pub struct InputManager {
    keyboard: KeyboardState,
    mouse: MouseState,
    gamepad: GamepadManager,
    action_map: InputActionMap,
    events: Vec<InputEvent>,
}

impl InputManager {
    pub fn new() -> Self {
        Self {
            keyboard: KeyboardState::new(),
            mouse: MouseState::new(),
            gamepad: GamepadManager::default(),
            action_map: InputActionMap::default_game_controls(),
            events: Vec::new(),
        }
    }

    /// Get keyboard state
    pub fn keyboard(&self) -> &KeyboardState {
        &self.keyboard
    }

    /// Get mouse state
    pub fn mouse(&self) -> &MouseState {
        &self.mouse
    }

    /// Get gamepad by ID
    pub fn gamepad(&self, id: GamepadId) -> Option<&super::GamepadState> {
        self.gamepad.gamepad(id)
    }

    /// Get first connected gamepad (convenience)
    pub fn first_gamepad(&self) -> Option<&super::GamepadState> {
        self.gamepad.first_gamepad()
    }

    /// Get all gamepads
    pub fn gamepads(&self) -> impl Iterator<Item = &super::GamepadState> {
        self.gamepad.gamepads()
    }

    /// Get action map
    pub fn action_map(&self) -> &InputActionMap {
        &self.action_map
    }

    /// Get mutable action map
    pub fn action_map_mut(&mut self) -> &mut InputActionMap {
        &mut self.action_map
    }

    /// Set action map
    pub fn set_action_map(&mut self, map: InputActionMap) {
        self.action_map = map;
    }

    /// Check if an action is currently active (pressed)
    pub fn is_action_active(&self, action: &InputAction) -> bool {
        let bindings = self.action_map.get_bindings(action);

        for binding in bindings {
            // Check modifiers first
            let modifiers_pressed = binding.modifiers.iter()
                .all(|modifier| self.keyboard.is_pressed(modifier.0));

            if !binding.modifiers.is_empty() && !modifiers_pressed {
                continue;
            }

            match &binding.binding {
                BindingType::Key(key) => {
                    if self.keyboard.is_pressed(key.0) {
                        return true;
                    }
                }
                BindingType::MouseButton(button) => {
                    if self.mouse.is_pressed(*button) {
                        return true;
                    }
                }
                BindingType::GamepadButton(button) => {
                    if let Some(gamepad) = self.first_gamepad() {
                        if gamepad.is_button_pressed(*button) {
                            return true;
                        }
                    }
                }
                BindingType::GamepadAxis { axis, threshold, positive } => {
                    if let Some(gamepad) = self.first_gamepad() {
                        let value = gamepad.axis_value(*axis);
                        if *positive && value > *threshold {
                            return true;
                        } else if !positive && value < -*threshold {
                            return true;
                        }
                    }
                }
                BindingType::MouseAxis { .. } => {
                    // Mouse axis is continuous, not binary
                    continue;
                }
            }
        }

        false
    }

    /// Check if an action was just pressed this frame
    pub fn is_action_just_pressed(&self, action: &InputAction) -> bool {
        let bindings = self.action_map.get_bindings(action);

        for binding in bindings {
            let modifiers_pressed = binding.modifiers.iter()
                .all(|modifier| self.keyboard.is_pressed(modifier.0));

            if !binding.modifiers.is_empty() && !modifiers_pressed {
                continue;
            }

            match &binding.binding {
                BindingType::Key(key) => {
                    if self.keyboard.just_pressed(key.0) {
                        return true;
                    }
                }
                BindingType::MouseButton(button) => {
                    if self.mouse.just_pressed(*button) {
                        return true;
                    }
                }
                BindingType::GamepadButton(_) => {
                    // Would need "just pressed" tracking for gamepad
                    // For now, not supported
                    continue;
                }
                _ => continue,
            }
        }

        false
    }

    /// Get action axis value (-1.0 to 1.0)
    pub fn get_action_axis(&self, action: &InputAction) -> AxisValue {
        let bindings = self.action_map.get_bindings(action);

        for binding in bindings {
            match &binding.binding {
                BindingType::GamepadAxis { axis, positive, .. } => {
                    if let Some(gamepad) = self.first_gamepad() {
                        let value = gamepad.axis_value(*axis);
                        return if *positive { value } else { -value };
                    }
                }
                BindingType::MouseAxis { horizontal } => {
                    let delta = self.mouse.delta();
                    return if *horizontal { delta.x } else { delta.y };
                }
                BindingType::Key(key) => {
                    if self.keyboard.is_pressed(key.0) {
                        return 1.0;
                    }
                }
                _ => continue,
            }
        }

        0.0
    }

    /// Get movement input as 2D vector (common for WASD/stick input)
    pub fn get_movement_vector(&self) -> Vec2 {
        let x = self.get_action_axis(&InputAction::new("MoveRight"))
            - self.get_action_axis(&InputAction::new("MoveLeft"));
        let y = self.get_action_axis(&InputAction::new("MoveForward"))
            - self.get_action_axis(&InputAction::new("MoveBackward"));

        Vec2::new(x, y)
    }

    /// Get look input as 2D vector (mouse/right stick)
    pub fn get_look_vector(&self) -> Vec2 {
        let x = self.get_action_axis(&InputAction::new("LookHorizontal"));
        let y = self.get_action_axis(&InputAction::new("LookVertical"));

        Vec2::new(x, y)
    }

    /// Process winit window event
    pub fn process_window_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::KeyboardInput { event, .. } => {
                if let winit::keyboard::PhysicalKey::Code(key_code) = event.physical_key {
                    match event.state {
                        ElementState::Pressed => {
                            self.keyboard.press_key(key_code);
                            self.events.push(InputEvent::KeyPressed(key_code));
                        }
                        ElementState::Released => {
                            self.keyboard.release_key(key_code);
                            self.events.push(InputEvent::KeyReleased(key_code));
                        }
                    }
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let mouse_button = (*button).into();
                match state {
                    ElementState::Pressed => {
                        self.mouse.press_button(mouse_button);
                        self.events.push(InputEvent::MouseButtonPressed(mouse_button));
                    }
                    ElementState::Released => {
                        self.mouse.release_button(mouse_button);
                        self.events.push(InputEvent::MouseButtonReleased(mouse_button));
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse.set_position(position.x as f32, position.y as f32);
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let scroll_delta = match delta {
                    MouseScrollDelta::LineDelta(_x, y) => *y,
                    MouseScrollDelta::PixelDelta(pos) => pos.y as f32 / 100.0,
                };
                self.mouse.add_scroll(scroll_delta);
                self.events.push(InputEvent::MouseScrolled { delta: scroll_delta });
            }
            _ => {}
        }
    }

    /// Process winit device event (for raw mouse input)
    pub fn process_device_event(&mut self, event: &DeviceEvent) {
        match event {
            DeviceEvent::MouseMotion { delta } => {
                self.mouse.add_delta(delta.0 as f32, delta.1 as f32);
                self.events.push(InputEvent::MouseMoved {
                    delta_x: delta.0 as f32,
                    delta_y: delta.1 as f32,
                });
            }
            _ => {}
        }
    }

    /// Update input state (call once per frame, after processing events)
    pub fn update(&mut self) {
        // Update gamepad states
        let gamepad_events = self.gamepad.update();
        self.events.extend(gamepad_events);

        // Clear frame-specific state
        self.keyboard.clear_frame_state();
        self.mouse.clear_frame_state();
    }

    /// Get input events from this frame
    pub fn events(&self) -> &[InputEvent] {
        &self.events
    }

    /// Clear events (call after processing)
    pub fn clear_events(&mut self) {
        self.events.clear();
    }

    /// Reset all input state
    pub fn reset(&mut self) {
        self.keyboard.reset();
        self.mouse.reset();
        self.events.clear();
    }
}

impl Default for InputManager {
    fn default() -> Self {
        Self::new()
    }
}
