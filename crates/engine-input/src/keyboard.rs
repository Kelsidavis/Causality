// Keyboard input handling

use std::collections::HashSet;

// Re-export winit's KeyCode for convenience
pub use winit::keyboard::KeyCode;

/// Keyboard input state tracker
#[derive(Debug, Default)]
pub struct KeyboardState {
    /// Keys currently pressed
    pressed: HashSet<KeyCode>,
    /// Keys pressed this frame
    just_pressed: HashSet<KeyCode>,
    /// Keys released this frame
    just_released: HashSet<KeyCode>,
}

impl KeyboardState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if a key is currently pressed
    pub fn is_pressed(&self, key: KeyCode) -> bool {
        self.pressed.contains(&key)
    }

    /// Check if a key was pressed this frame
    pub fn just_pressed(&self, key: KeyCode) -> bool {
        self.just_pressed.contains(&key)
    }

    /// Check if a key was released this frame
    pub fn just_released(&self, key: KeyCode) -> bool {
        self.just_released.contains(&key)
    }

    /// Get all currently pressed keys
    pub fn pressed_keys(&self) -> impl Iterator<Item = &KeyCode> {
        self.pressed.iter()
    }

    /// Update keyboard state with a key press
    pub(crate) fn press_key(&mut self, key: KeyCode) {
        if !self.pressed.contains(&key) {
            self.just_pressed.insert(key);
        }
        self.pressed.insert(key);
    }

    /// Update keyboard state with a key release
    pub(crate) fn release_key(&mut self, key: KeyCode) {
        if self.pressed.remove(&key) {
            self.just_released.insert(key);
        }
    }

    /// Clear frame-specific state (call at end of frame)
    pub(crate) fn clear_frame_state(&mut self) {
        self.just_pressed.clear();
        self.just_released.clear();
    }

    /// Reset all keyboard state
    pub fn reset(&mut self) {
        self.pressed.clear();
        self.just_pressed.clear();
        self.just_released.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyboard_state() {
        let mut state = KeyboardState::new();

        // Initially nothing pressed
        assert!(!state.is_pressed(KeyCode::KeyW));
        assert!(!state.just_pressed(KeyCode::KeyW));

        // Press W
        state.press_key(KeyCode::KeyW);
        assert!(state.is_pressed(KeyCode::KeyW));
        assert!(state.just_pressed(KeyCode::KeyW));

        // Clear frame state
        state.clear_frame_state();
        assert!(state.is_pressed(KeyCode::KeyW));
        assert!(!state.just_pressed(KeyCode::KeyW));

        // Release W
        state.release_key(KeyCode::KeyW);
        assert!(!state.is_pressed(KeyCode::KeyW));
        assert!(state.just_released(KeyCode::KeyW));

        // Clear frame state
        state.clear_frame_state();
        assert!(!state.just_released(KeyCode::KeyW));
    }
}
