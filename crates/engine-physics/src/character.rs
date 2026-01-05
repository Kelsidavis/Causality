// Character controller for player movement

use engine_scene::entity::Component;
use engine_scene::impl_component;
use glam::Vec3;
use serde::{Deserialize, Serialize};
use std::any::Any;

/// Character controller component for FPS/TPS movement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterController {
    /// Movement speed in units per second
    pub move_speed: f32,
    /// Sprint multiplier
    pub sprint_multiplier: f32,
    /// Jump force
    pub jump_force: f32,
    /// Whether the character is on the ground
    pub grounded: bool,
    /// Gravity multiplier
    pub gravity_scale: f32,
    /// Ground check distance
    pub ground_distance: f32,
    /// Current velocity (for smooth movement)
    pub velocity: Vec3,
    /// Air control factor (0.0 = no air control, 1.0 = full control)
    pub air_control: f32,
}

impl CharacterController {
    pub fn new() -> Self {
        Self {
            move_speed: 5.0,
            sprint_multiplier: 1.5,
            jump_force: 10.0,
            grounded: false,
            gravity_scale: 1.0,
            ground_distance: 0.1,
            velocity: Vec3::ZERO,
            air_control: 0.3,
        }
    }

    pub fn with_speed(mut self, speed: f32) -> Self {
        self.move_speed = speed;
        self
    }

    pub fn with_jump_force(mut self, force: f32) -> Self {
        self.jump_force = force;
        self
    }

    /// Calculate movement for this frame
    pub fn calculate_movement(&mut self, input: Vec3, delta_time: f32, is_sprinting: bool) -> Vec3 {
        let speed = if is_sprinting {
            self.move_speed * self.sprint_multiplier
        } else {
            self.move_speed
        };

        let control_factor = if self.grounded {
            1.0
        } else {
            self.air_control
        };

        // Smooth movement
        let target_velocity = input.normalize_or_zero() * speed;
        self.velocity = self.velocity.lerp(target_velocity, control_factor * delta_time * 10.0);

        self.velocity * delta_time
    }

    /// Initiate a jump
    pub fn jump(&mut self) -> Option<f32> {
        if self.grounded {
            self.grounded = false;
            Some(self.jump_force)
        } else {
            None
        }
    }
}

impl Default for CharacterController {
    fn default() -> Self {
        Self::new()
    }
}

impl_component!(CharacterController);
