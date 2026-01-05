// Audio Listener - represents the player's ears (typically camera position)

use glam::{Quat, Vec3};

/// Audio listener - represents where the player hears sound from
#[derive(Debug, Clone)]
pub struct AudioListener {
    /// Position in world space
    pub position: Vec3,
    /// Forward direction
    pub forward: Vec3,
    /// Up direction
    pub up: Vec3,
}

impl AudioListener {
    /// Create a new audio listener
    pub fn new(position: Vec3, forward: Vec3, up: Vec3) -> Self {
        Self {
            position,
            forward: forward.normalize(),
            up: up.normalize(),
        }
    }

    /// Create from position and rotation
    pub fn from_transform(position: Vec3, rotation: Quat) -> Self {
        let forward = rotation * Vec3::NEG_Z;
        let up = rotation * Vec3::Y;
        Self::new(position, forward, up)
    }

    /// Update from transform
    pub fn update_from_transform(&mut self, position: Vec3, rotation: Quat) {
        self.position = position;
        self.forward = (rotation * Vec3::NEG_Z).normalize();
        self.up = (rotation * Vec3::Y).normalize();
    }
}

impl Default for AudioListener {
    fn default() -> Self {
        Self::new(Vec3::ZERO, Vec3::NEG_Z, Vec3::Y)
    }
}
