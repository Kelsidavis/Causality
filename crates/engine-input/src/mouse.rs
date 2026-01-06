// Mouse input handling

use std::collections::HashSet;
use glam::Vec2;

/// Mouse buttons
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Other(u16),
}

impl From<winit::event::MouseButton> for MouseButton {
    fn from(button: winit::event::MouseButton) -> Self {
        match button {
            winit::event::MouseButton::Left => MouseButton::Left,
            winit::event::MouseButton::Right => MouseButton::Right,
            winit::event::MouseButton::Middle => MouseButton::Middle,
            winit::event::MouseButton::Back => MouseButton::Other(3),
            winit::event::MouseButton::Forward => MouseButton::Other(4),
            winit::event::MouseButton::Other(id) => MouseButton::Other(id),
        }
    }
}

/// Mouse input state tracker
#[derive(Debug)]
pub struct MouseState {
    /// Current mouse position (screen coordinates)
    position: Vec2,
    /// Mouse position in previous frame
    previous_position: Vec2,
    /// Mouse delta this frame
    delta: Vec2,
    /// Scroll wheel delta this frame
    scroll_delta: f32,
    /// Buttons currently pressed
    pressed: HashSet<MouseButton>,
    /// Buttons pressed this frame
    just_pressed: HashSet<MouseButton>,
    /// Buttons released this frame
    just_released: HashSet<MouseButton>,
    /// Mouse sensitivity multiplier
    sensitivity: f32,
}

impl Default for MouseState {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            previous_position: Vec2::ZERO,
            delta: Vec2::ZERO,
            scroll_delta: 0.0,
            pressed: HashSet::new(),
            just_pressed: HashSet::new(),
            just_released: HashSet::new(),
            sensitivity: 1.0,
        }
    }
}

impl MouseState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get current mouse position
    pub fn position(&self) -> Vec2 {
        self.position
    }

    /// Get mouse movement delta this frame
    pub fn delta(&self) -> Vec2 {
        self.delta * self.sensitivity
    }

    /// Get scroll wheel delta this frame
    pub fn scroll_delta(&self) -> f32 {
        self.scroll_delta
    }

    /// Check if a mouse button is currently pressed
    pub fn is_pressed(&self, button: MouseButton) -> bool {
        self.pressed.contains(&button)
    }

    /// Check if a mouse button was pressed this frame
    pub fn just_pressed(&self, button: MouseButton) -> bool {
        self.just_pressed.contains(&button)
    }

    /// Check if a mouse button was released this frame
    pub fn just_released(&self, button: MouseButton) -> bool {
        self.just_released.contains(&button)
    }

    /// Set mouse sensitivity (default 1.0)
    pub fn set_sensitivity(&mut self, sensitivity: f32) {
        self.sensitivity = sensitivity;
    }

    /// Get mouse sensitivity
    pub fn sensitivity(&self) -> f32 {
        self.sensitivity
    }

    /// Update mouse position
    pub(crate) fn set_position(&mut self, x: f32, y: f32) {
        self.previous_position = self.position;
        self.position = Vec2::new(x, y);
    }

    /// Update mouse delta (for captured mouse mode)
    pub(crate) fn add_delta(&mut self, dx: f32, dy: f32) {
        self.delta += Vec2::new(dx, dy);
    }

    /// Update scroll delta
    pub(crate) fn add_scroll(&mut self, delta: f32) {
        self.scroll_delta += delta;
    }

    /// Press mouse button
    pub(crate) fn press_button(&mut self, button: MouseButton) {
        if !self.pressed.contains(&button) {
            self.just_pressed.insert(button);
        }
        self.pressed.insert(button);
    }

    /// Release mouse button
    pub(crate) fn release_button(&mut self, button: MouseButton) {
        if self.pressed.remove(&button) {
            self.just_released.insert(button);
        }
    }

    /// Clear frame-specific state (call at end of frame)
    pub(crate) fn clear_frame_state(&mut self) {
        self.delta = self.position - self.previous_position;
        self.previous_position = self.position;
        self.scroll_delta = 0.0;
        self.just_pressed.clear();
        self.just_released.clear();
    }

    /// Reset all mouse state
    pub fn reset(&mut self) {
        self.position = Vec2::ZERO;
        self.previous_position = Vec2::ZERO;
        self.delta = Vec2::ZERO;
        self.scroll_delta = 0.0;
        self.pressed.clear();
        self.just_pressed.clear();
        self.just_released.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mouse_state() {
        let mut state = MouseState::new();

        // Press button
        state.press_button(MouseButton::Left);
        assert!(state.is_pressed(MouseButton::Left));
        assert!(state.just_pressed(MouseButton::Left));

        // Clear frame
        state.clear_frame_state();
        assert!(state.is_pressed(MouseButton::Left));
        assert!(!state.just_pressed(MouseButton::Left));

        // Release button
        state.release_button(MouseButton::Left);
        assert!(!state.is_pressed(MouseButton::Left));
        assert!(state.just_released(MouseButton::Left));
    }

    #[test]
    fn test_mouse_position() {
        let mut state = MouseState::new();

        state.set_position(100.0, 200.0);
        assert_eq!(state.position(), Vec2::new(100.0, 200.0));

        state.set_position(150.0, 250.0);
        state.clear_frame_state();
        assert_eq!(state.delta(), Vec2::new(50.0, 50.0));
    }
}
