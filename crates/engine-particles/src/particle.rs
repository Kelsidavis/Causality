// Individual particle data and settings

use glam::{Vec3, Vec4};

/// A single particle instance
#[derive(Debug, Clone)]
pub struct Particle {
    /// Current position
    pub position: Vec3,
    /// Current velocity
    pub velocity: Vec3,
    /// Current color (RGBA)
    pub color: Vec4,
    /// Current size
    pub size: f32,
    /// Current lifetime (seconds remaining)
    pub lifetime: f32,
    /// Initial lifetime (for interpolation)
    pub initial_lifetime: f32,
}

impl Particle {
    /// Create a new particle
    pub fn new(position: Vec3, velocity: Vec3, color: Vec4, size: f32, lifetime: f32) -> Self {
        Self {
            position,
            velocity,
            color,
            size,
            lifetime,
            initial_lifetime: lifetime,
        }
    }

    /// Update particle (returns false if particle is dead)
    pub fn update(&mut self, delta_time: f32, gravity: Vec3) -> bool {
        self.lifetime -= delta_time;

        if self.lifetime <= 0.0 {
            return false;
        }

        // Apply gravity
        self.velocity += gravity * delta_time;

        // Update position
        self.position += self.velocity * delta_time;

        true
    }

    /// Get normalized lifetime (0.0 = just spawned, 1.0 = about to die)
    pub fn normalized_age(&self) -> f32 {
        1.0 - (self.lifetime / self.initial_lifetime).clamp(0.0, 1.0)
    }
}

/// Particle settings for emitters
#[derive(Debug, Clone)]
pub struct ParticleSettings {
    /// Lifetime range (min, max) in seconds
    pub lifetime_range: (f32, f32),
    /// Initial size range (min, max)
    pub size_range: (f32, f32),
    /// Initial speed range (min, max)
    pub speed_range: (f32, f32),
    /// Color gradient (interpolated over lifetime)
    pub color_gradient: Vec<Vec4>,
    /// Size curve (interpolated over lifetime)
    pub size_curve: Vec<f32>,
    /// Gravity affecting particles
    pub gravity: Vec3,
}

impl Default for ParticleSettings {
    fn default() -> Self {
        Self {
            lifetime_range: (1.0, 2.0),
            size_range: (0.1, 0.2),
            speed_range: (1.0, 3.0),
            color_gradient: vec![
                Vec4::new(1.0, 1.0, 1.0, 1.0),
                Vec4::new(1.0, 1.0, 1.0, 0.0),
            ],
            size_curve: vec![1.0, 0.0],
            gravity: Vec3::new(0.0, -9.81, 0.0),
        }
    }
}

impl ParticleSettings {
    /// Sample color from gradient based on normalized age
    pub fn sample_color(&self, normalized_age: f32) -> Vec4 {
        if self.color_gradient.is_empty() {
            return Vec4::ONE;
        }

        if self.color_gradient.len() == 1 {
            return self.color_gradient[0];
        }

        let t = normalized_age.clamp(0.0, 1.0);
        let segment_count = self.color_gradient.len() - 1;
        let segment = (t * segment_count as f32).floor() as usize;
        let segment = segment.min(segment_count - 1);

        let local_t = (t * segment_count as f32) - segment as f32;
        self.color_gradient[segment].lerp(self.color_gradient[segment + 1], local_t)
    }

    /// Sample size from curve based on normalized age
    pub fn sample_size(&self, normalized_age: f32, initial_size: f32) -> f32 {
        if self.size_curve.is_empty() {
            return initial_size;
        }

        if self.size_curve.len() == 1 {
            return initial_size * self.size_curve[0];
        }

        let t = normalized_age.clamp(0.0, 1.0);
        let segment_count = self.size_curve.len() - 1;
        let segment = (t * segment_count as f32).floor() as usize;
        let segment = segment.min(segment_count - 1);

        let local_t = (t * segment_count as f32) - segment as f32;
        let scale = self.size_curve[segment] + (self.size_curve[segment + 1] - self.size_curve[segment]) * local_t;
        initial_size * scale
    }
}
