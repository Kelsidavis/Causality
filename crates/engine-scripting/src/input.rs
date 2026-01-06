// Input API for scripts

use engine_input::{InputManager, InputAction, KeyCode, MouseButton};
use rhai::{Engine, Dynamic};
use std::sync::{Arc, Mutex};

/// Shared input manager for scripts
pub type SharedInputManager = Arc<Mutex<InputManager>>;

/// Register input functions with Rhai engine
pub fn register_input_api(engine: &mut Engine, input_manager: SharedInputManager) {
    // Clone for each closure
    let input_clone1 = input_manager.clone();
    let input_clone2 = input_manager.clone();
    let input_clone3 = input_manager.clone();
    let input_clone4 = input_manager.clone();
    let input_clone5 = input_manager.clone();
    let input_clone6 = input_manager.clone();
    let input_clone7 = input_manager.clone();
    let input_clone8 = input_manager.clone();
    let input_clone9 = input_manager.clone();

    // Action-based input (high-level API)
    engine
        .register_fn("is_action_active", move |action: &str| {
            let input = input_clone1.lock().unwrap();
            input.is_action_active(&InputAction::new(action))
        })
        .register_fn("is_action_just_pressed", move |action: &str| {
            let input = input_clone2.lock().unwrap();
            input.is_action_just_pressed(&InputAction::new(action))
        })
        .register_fn("get_action_axis", move |action: &str| {
            let input = input_clone3.lock().unwrap();
            input.get_action_axis(&InputAction::new(action))
        });

    // Movement/Look helpers
    engine
        .register_fn("get_movement", move || {
            let input = input_clone4.lock().unwrap();
            let movement = input.get_movement_vector();
            // Return as array for Rhai
            Dynamic::from([movement.x, movement.y])
        })
        .register_fn("get_look", move || {
            let input = input_clone5.lock().unwrap();
            let look = input.get_look_vector();
            Dynamic::from([look.x, look.y])
        });

    // Direct input access (low-level API)
    engine
        .register_fn("is_key_pressed", move |key: &str| {
            let input = input_clone6.lock().unwrap();
            // Map common key names to KeyCode
            let key_code = match key.to_lowercase().as_str() {
                "w" => Some(KeyCode::KeyW),
                "a" => Some(KeyCode::KeyA),
                "s" => Some(KeyCode::KeyS),
                "d" => Some(KeyCode::KeyD),
                "space" => Some(KeyCode::Space),
                "shift" => Some(KeyCode::ShiftLeft),
                "ctrl" => Some(KeyCode::ControlLeft),
                "escape" => Some(KeyCode::Escape),
                "enter" => Some(KeyCode::Enter),
                "tab" => Some(KeyCode::Tab),
                _ => None,
            };

            if let Some(code) = key_code {
                input.keyboard().is_pressed(code)
            } else {
                false
            }
        })
        .register_fn("is_mouse_button_pressed", move |button: &str| {
            let input = input_clone7.lock().unwrap();
            let mouse_button = match button.to_lowercase().as_str() {
                "left" => Some(MouseButton::Left),
                "right" => Some(MouseButton::Right),
                "middle" => Some(MouseButton::Middle),
                _ => None,
            };

            if let Some(btn) = mouse_button {
                input.mouse().is_pressed(btn)
            } else {
                false
            }
        })
        .register_fn("get_mouse_delta", move || {
            let input = input_clone8.lock().unwrap();
            let delta = input.mouse().delta();
            Dynamic::from([delta.x, delta.y])
        })
        .register_fn("get_mouse_scroll", move || {
            let input = input_clone9.lock().unwrap();
            input.mouse().scroll_delta()
        });
}

/*
Example script functions:

```rhai
// Check if jump is pressed
if is_action_active("Jump") {
    print("Jumping!");
}

// Check for one-frame press
if is_action_just_pressed("Fire") {
    print("Fired weapon!");
}

// Get movement vector
let movement = get_movement();
let move_x = movement[0];
let move_y = movement[1];

// Get look input
let look = get_look();
let look_x = look[0];
let look_y = look[1];

// Direct key check
if is_key_pressed("w") {
    print("W key is down");
}

// Mouse input
if is_mouse_button_pressed("left") {
    print("Left mouse button pressed");
}

let mouse_delta = get_mouse_delta();
let scroll = get_mouse_scroll();
```
*/
