// Input action mapping system for rebindable controls

use std::collections::HashMap;
use serde::{Deserialize, Serialize, Serializer, Deserializer};
use super::{MouseButton, GamepadButton, GamepadAxis, GamepadId};
use winit::keyboard::KeyCode;

// Serializable wrapper for KeyCode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SerializableKeyCode(pub KeyCode);

impl Serialize for SerializableKeyCode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Serialize as string representation
        serializer.serialize_str(&format!("{:?}", self.0))
    }
}

impl<'de> Deserialize<'de> for SerializableKeyCode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let _s = String::deserialize(deserializer)?;
        // For now, just store as Space if we can't parse
        // In a production system, you'd want a proper mapping
        // TODO: Implement proper string-to-KeyCode mapping
        Ok(SerializableKeyCode(KeyCode::Space))
    }
}

impl From<KeyCode> for SerializableKeyCode {
    fn from(key: KeyCode) -> Self {
        SerializableKeyCode(key)
    }
}

impl From<SerializableKeyCode> for KeyCode {
    fn from(key: SerializableKeyCode) -> Self {
        key.0
    }
}

/// Named input action (e.g., "Jump", "Fire", "MoveForward")
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InputAction(pub String);

impl InputAction {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }
}

impl From<&str> for InputAction {
    fn from(s: &str) -> Self {
        InputAction(s.to_string())
    }
}

/// Type of input binding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BindingType {
    /// Keyboard key
    Key(SerializableKeyCode),
    /// Mouse button
    MouseButton(MouseButton),
    /// Gamepad button
    GamepadButton(GamepadButton),
    /// Gamepad axis (with threshold for button-like behavior)
    GamepadAxis {
        axis: GamepadAxis,
        threshold: f32,
        positive: bool, // true = positive direction, false = negative direction
    },
    /// Mouse axis (X or Y)
    MouseAxis {
        horizontal: bool, // true = X axis, false = Y axis
    },
}

/// Input action binding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionBinding {
    pub action: InputAction,
    pub binding: BindingType,
    /// Modifier keys required (e.g., Ctrl+C)
    pub modifiers: Vec<SerializableKeyCode>,
}

impl ActionBinding {
    pub fn new(action: impl Into<InputAction>, binding: BindingType) -> Self {
        Self {
            action: action.into(),
            binding,
            modifiers: Vec::new(),
        }
    }

    pub fn with_modifier(mut self, modifier: KeyCode) -> Self {
        self.modifiers.push(SerializableKeyCode(modifier));
        self
    }
}

/// Input action map - maps actions to input bindings
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct InputActionMap {
    bindings: Vec<ActionBinding>,
    action_to_bindings: HashMap<String, Vec<usize>>,
}

impl InputActionMap {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a binding for an action
    pub fn bind(&mut self, binding: ActionBinding) {
        let action_name = binding.action.0.clone();
        let index = self.bindings.len();
        self.bindings.push(binding);

        self.action_to_bindings
            .entry(action_name)
            .or_insert_with(Vec::new)
            .push(index);
    }

    /// Add multiple bindings at once
    pub fn bind_many(&mut self, bindings: Vec<ActionBinding>) {
        for binding in bindings {
            self.bind(binding);
        }
    }

    /// Remove all bindings for an action
    pub fn unbind_action(&mut self, action: &InputAction) {
        if let Some(indices) = self.action_to_bindings.remove(&action.0) {
            // Mark bindings as removed (don't actually remove to keep indices stable)
            for idx in indices {
                if let Some(binding) = self.bindings.get_mut(idx) {
                    // Replace with a dummy binding
                    *binding = ActionBinding::new("__removed__", BindingType::Key(SerializableKeyCode(KeyCode::F24)));
                }
            }
        }
    }

    /// Get all bindings for an action
    pub fn get_bindings(&self, action: &InputAction) -> Vec<&ActionBinding> {
        self.action_to_bindings
            .get(&action.0)
            .map(|indices| {
                indices
                    .iter()
                    .filter_map(|&idx| self.bindings.get(idx))
                    .filter(|b| b.action.0 != "__removed__")
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all actions
    pub fn actions(&self) -> impl Iterator<Item = &String> {
        self.action_to_bindings.keys()
    }

    /// Clear all bindings
    pub fn clear(&mut self) {
        self.bindings.clear();
        self.action_to_bindings.clear();
    }

    /// Create default action map for common game controls
    pub fn default_game_controls() -> Self {
        let mut map = Self::new();

        // Movement
        map.bind(ActionBinding::new("MoveForward", BindingType::Key(SerializableKeyCode(KeyCode::KeyW))));
        map.bind(ActionBinding::new("MoveForward", BindingType::GamepadAxis {
            axis: GamepadAxis::LeftStickY,
            threshold: 0.3,
            positive: true,
        }));

        map.bind(ActionBinding::new("MoveBackward", BindingType::Key(SerializableKeyCode(KeyCode::KeyS))));
        map.bind(ActionBinding::new("MoveBackward", BindingType::GamepadAxis {
            axis: GamepadAxis::LeftStickY,
            threshold: 0.3,
            positive: false,
        }));

        map.bind(ActionBinding::new("MoveLeft", BindingType::Key(SerializableKeyCode(KeyCode::KeyA))));
        map.bind(ActionBinding::new("MoveLeft", BindingType::GamepadAxis {
            axis: GamepadAxis::LeftStickX,
            threshold: 0.3,
            positive: false,
        }));

        map.bind(ActionBinding::new("MoveRight", BindingType::Key(SerializableKeyCode(KeyCode::KeyD))));
        map.bind(ActionBinding::new("MoveRight", BindingType::GamepadAxis {
            axis: GamepadAxis::LeftStickX,
            threshold: 0.3,
            positive: true,
        }));

        // Actions
        map.bind(ActionBinding::new("Jump", BindingType::Key(SerializableKeyCode(KeyCode::Space))));
        map.bind(ActionBinding::new("Jump", BindingType::GamepadButton(GamepadButton::South)));

        map.bind(ActionBinding::new("Fire", BindingType::MouseButton(MouseButton::Left)));
        map.bind(ActionBinding::new("Fire", BindingType::GamepadButton(GamepadButton::RightTrigger2)));

        map.bind(ActionBinding::new("AimDownSights", BindingType::MouseButton(MouseButton::Right)));
        map.bind(ActionBinding::new("AimDownSights", BindingType::GamepadButton(GamepadButton::LeftTrigger2)));

        map.bind(ActionBinding::new("Reload", BindingType::Key(SerializableKeyCode(KeyCode::KeyR))));
        map.bind(ActionBinding::new("Reload", BindingType::GamepadButton(GamepadButton::West)));

        map.bind(ActionBinding::new("Interact", BindingType::Key(SerializableKeyCode(KeyCode::KeyE))));
        map.bind(ActionBinding::new("Interact", BindingType::GamepadButton(GamepadButton::East)));

        // Camera/Look
        map.bind(ActionBinding::new("LookHorizontal", BindingType::MouseAxis { horizontal: true }));
        map.bind(ActionBinding::new("LookHorizontal", BindingType::GamepadAxis {
            axis: GamepadAxis::RightStickX,
            threshold: 0.0,
            positive: true,
        }));

        map.bind(ActionBinding::new("LookVertical", BindingType::MouseAxis { horizontal: false }));
        map.bind(ActionBinding::new("LookVertical", BindingType::GamepadAxis {
            axis: GamepadAxis::RightStickY,
            threshold: 0.0,
            positive: true,
        }));

        // UI
        map.bind(ActionBinding::new("Pause", BindingType::Key(SerializableKeyCode(KeyCode::Escape))));
        map.bind(ActionBinding::new("Pause", BindingType::GamepadButton(GamepadButton::Start)));

        map.bind(ActionBinding::new("Menu", BindingType::Key(SerializableKeyCode(KeyCode::Tab))));
        map.bind(ActionBinding::new("Menu", BindingType::GamepadButton(GamepadButton::Select)));

        map
    }

    /// Create default action map for editor controls
    pub fn default_editor_controls() -> Self {
        let mut map = Self::new();

        // Editor camera
        map.bind(ActionBinding::new("EditorMoveForward", BindingType::Key(SerializableKeyCode(KeyCode::KeyW))));
        map.bind(ActionBinding::new("EditorMoveBackward", BindingType::Key(SerializableKeyCode(KeyCode::KeyS))));
        map.bind(ActionBinding::new("EditorMoveLeft", BindingType::Key(SerializableKeyCode(KeyCode::KeyA))));
        map.bind(ActionBinding::new("EditorMoveRight", BindingType::Key(SerializableKeyCode(KeyCode::KeyD))));
        map.bind(ActionBinding::new("EditorMoveUp", BindingType::Key(SerializableKeyCode(KeyCode::KeyE))));
        map.bind(ActionBinding::new("EditorMoveDown", BindingType::Key(SerializableKeyCode(KeyCode::KeyQ))));

        // Editor actions
        map.bind(ActionBinding::new("EditorSelect", BindingType::MouseButton(MouseButton::Left)));
        map.bind(ActionBinding::new("EditorRotateCamera", BindingType::MouseButton(MouseButton::Right)));
        map.bind(ActionBinding::new("EditorPanCamera", BindingType::MouseButton(MouseButton::Middle)));

        // File operations
        map.bind(ActionBinding::new("Save", BindingType::Key(SerializableKeyCode(KeyCode::KeyS)))
            .with_modifier(KeyCode::ControlLeft));
        map.bind(ActionBinding::new("Open", BindingType::Key(SerializableKeyCode(KeyCode::KeyO)))
            .with_modifier(KeyCode::ControlLeft));
        map.bind(ActionBinding::new("New", BindingType::Key(SerializableKeyCode(KeyCode::KeyN)))
            .with_modifier(KeyCode::ControlLeft));

        // Edit operations
        map.bind(ActionBinding::new("Undo", BindingType::Key(SerializableKeyCode(KeyCode::KeyZ)))
            .with_modifier(KeyCode::ControlLeft));
        map.bind(ActionBinding::new("Redo", BindingType::Key(SerializableKeyCode(KeyCode::KeyY)))
            .with_modifier(KeyCode::ControlLeft));
        map.bind(ActionBinding::new("Delete", BindingType::Key(SerializableKeyCode(KeyCode::Delete))));
        map.bind(ActionBinding::new("Duplicate", BindingType::Key(SerializableKeyCode(KeyCode::KeyD)))
            .with_modifier(KeyCode::ControlLeft));

        map
    }

    /// Save action map to RON file
    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let ron_string = ron::ser::to_string_pretty(self, Default::default())?;
        std::fs::write(path, ron_string)?;
        Ok(())
    }

    /// Load action map from RON file
    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let ron_string = std::fs::read_to_string(path)?;
        let map: InputActionMap = ron::de::from_str(&ron_string)?;
        Ok(map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_binding() {
        let mut map = InputActionMap::new();

        map.bind(ActionBinding::new("Jump", BindingType::Key(SerializableKeyCode(KeyCode::Space))));
        map.bind(ActionBinding::new("Jump", BindingType::GamepadButton(GamepadButton::South)));

        let bindings = map.get_bindings(&InputAction::new("Jump"));
        assert_eq!(bindings.len(), 2);
    }

    #[test]
    fn test_unbind() {
        let mut map = InputActionMap::new();
        map.bind(ActionBinding::new("Jump", BindingType::Key(SerializableKeyCode(KeyCode::Space))));

        assert_eq!(map.get_bindings(&InputAction::new("Jump")).len(), 1);

        map.unbind_action(&InputAction::new("Jump"));
        assert_eq!(map.get_bindings(&InputAction::new("Jump")).len(), 0);
    }
}
