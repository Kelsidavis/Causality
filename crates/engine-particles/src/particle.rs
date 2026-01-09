// Particle data structure for GPU

use bytemuck::{Pod, Zeroable};
use glam::Vec3;

/// GPU-compatible particle structure
/// Total size: 64 bytes (GPU-friendly alignment)
#[repr(C, align(16))]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct GpuParticle {
    /// Position in world space
    pub position: [f32; 3],
    pub _padding1: f32,

    /// Velocity vector
    pub velocity: [f32; 3],
    pub _padding2: f32,

    /// RGBA color with alpha
    pub color: [f32; 4],

    /// Current size
    pub size: f32,

    /// Current lifetime (seconds)
    pub lifetime: f32,

    /// Maximum lifetime (seconds)
    pub max_lifetime: f32,

    /// Rotation angle (radians)
    pub rotation: f32,
}

impl Default for GpuParticle {
    fn default() -> Self {
        Self {
            position: [0.0; 3],
            _padding1: 0.0,
            velocity: [0.0; 3],
            _padding2: 0.0,
            color: [1.0, 1.0, 1.0, 1.0],
            size: 1.0,
            lifetime: 0.0,
            max_lifetime: 1.0,
            rotation: 0.0,
        }
    }
}

impl GpuParticle {
    /// Create a new particle
    pub fn new(
        position: Vec3,
        velocity: Vec3,
        color: [f32; 4],
        size: f32,
        max_lifetime: f32,
    ) -> Self {
        Self {
            position: position.to_array(),
            _padding1: 0.0,
            velocity: velocity.to_array(),
            _padding2: 0.0,
            color,
            size,
            lifetime: 0.0,
            max_lifetime,
            rotation: 0.0,
        }
    }

    /// Check if particle is alive
    pub fn is_alive(&self) -> bool {
        self.lifetime < self.max_lifetime
    }

    /// Get life ratio (0.0 = just spawned, 1.0 = about to die)
    pub fn life_ratio(&self) -> f32 {
        (self.lifetime / self.max_lifetime).clamp(0.0, 1.0)
    }

    /// Mark particle as dead (move offscreen)
    pub fn kill(&mut self) {
        self.position = [0.0, -9999.0, 0.0];
        self.lifetime = self.max_lifetime + 1.0;
    }
}

// Verify size is exactly 64 bytes
const _: () = assert!(std::mem::size_of::<GpuParticle>() == 64);
