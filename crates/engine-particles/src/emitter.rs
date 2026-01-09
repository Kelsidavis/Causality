// Particle emitter shapes and properties

use glam::Vec3;
use serde::{Deserialize, Serialize};

/// Shape of the particle emitter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmitterShape {
    /// Emit from a single point
    Point,

    /// Emit from within a sphere
    Sphere { radius: f32 },

    /// Emit from within a cone
    Cone { angle: f32, radius: f32 },

    /// Emit from within a box
    Box { size: Vec3 },

    /// Emit from a flat circle
    Circle { radius: f32 },
}

impl Default for EmitterShape {
    fn default() -> Self {
        EmitterShape::Point
    }
}

impl EmitterShape {
    /// Get a random position within the emitter shape
    pub fn sample_position(&self) -> Vec3 {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        match self {
            EmitterShape::Point => Vec3::ZERO,

            EmitterShape::Sphere { radius } => {
                // Uniform sampling in sphere
                let theta = rng.gen_range(0.0..std::f32::consts::TAU);
                let phi = rng.gen_range(0.0..std::f32::consts::PI);
                let r = rng.gen_range(0.0..*radius);

                Vec3::new(
                    r * phi.sin() * theta.cos(),
                    r * phi.sin() * theta.sin(),
                    r * phi.cos(),
                )
            }

            EmitterShape::Cone { angle, radius } => {
                // Sample within cone
                let theta = rng.gen_range(0.0..std::f32::consts::TAU);
                let cone_height = rng.gen_range(0.0..*radius);
                let cone_radius = cone_height * angle.tan();
                let r = rng.gen_range(0.0..cone_radius);

                Vec3::new(r * theta.cos(), cone_height, r * theta.sin())
            }

            EmitterShape::Box { size } => {
                // Uniform sampling in box
                Vec3::new(
                    rng.gen_range(-size.x / 2.0..size.x / 2.0),
                    rng.gen_range(-size.y / 2.0..size.y / 2.0),
                    rng.gen_range(-size.z / 2.0..size.z / 2.0),
                )
            }

            EmitterShape::Circle { radius } => {
                // Uniform sampling in circle
                let theta = rng.gen_range(0.0..std::f32::consts::TAU);
                let r = rng.gen_range(0.0..*radius);

                Vec3::new(r * theta.cos(), 0.0, r * theta.sin())
            }
        }
    }
}

/// Emitter properties defining particle behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmitterProperties {
    /// Shape of emission
    pub shape: EmitterShape,

    /// Particles emitted per second
    pub rate: f32,

    /// Initial velocity direction
    pub initial_velocity: Vec3,

    /// Randomness in velocity (0.0 = no randomness, 1.0 = full random)
    pub velocity_randomness: f32,

    /// Particle lifetime in seconds
    pub lifetime: f32,

    /// Randomness in lifetime
    pub lifetime_randomness: f32,

    /// Initial particle size
    pub initial_size: f32,

    /// Size multipliers over lifetime [0.0, 0.25, 0.5, 0.75, 1.0]
    pub size_over_lifetime: Vec<f32>,

    /// Initial color (RGBA)
    pub initial_color: [f32; 4],

    /// Color gradient over lifetime
    pub color_over_lifetime: Vec<[f32; 4]>,

    /// Gravity applied to particles
    pub gravity: Vec3,
}

impl Default for EmitterProperties {
    fn default() -> Self {
        Self {
            shape: EmitterShape::Point,
            rate: 10.0,
            initial_velocity: Vec3::Y,
            velocity_randomness: 0.1,
            lifetime: 1.0,
            lifetime_randomness: 0.0,
            initial_size: 1.0,
            size_over_lifetime: vec![1.0, 0.8, 0.5, 0.2, 0.0],
            initial_color: [1.0, 1.0, 1.0, 1.0],
            color_over_lifetime: vec![
                [1.0, 1.0, 1.0, 1.0],
                [1.0, 1.0, 1.0, 0.5],
                [1.0, 1.0, 1.0, 0.0],
            ],
            gravity: Vec3::new(0.0, -9.81, 0.0),
        }
    }
}

impl EmitterProperties {
    /// Get initial velocity with randomness applied
    pub fn sample_velocity(&self) -> Vec3 {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        let base = self.initial_velocity;
        let randomness = self.velocity_randomness;

        if randomness == 0.0 {
            return base;
        }

        let random_vec = Vec3::new(
            rng.gen_range(-1.0..1.0),
            rng.gen_range(-1.0..1.0),
            rng.gen_range(-1.0..1.0),
        ) * randomness;

        base + random_vec
    }

    /// Sample lifetime with randomness
    pub fn sample_lifetime(&self) -> f32 {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        let base = self.lifetime;
        let randomness = self.lifetime_randomness;

        base + rng.gen_range(-randomness..randomness)
    }

    /// Evaluate size at given life ratio (0.0 to 1.0)
    pub fn evaluate_size(&self, life_ratio: f32) -> f32 {
        if self.size_over_lifetime.is_empty() {
            return self.initial_size;
        }

        let index = (life_ratio * (self.size_over_lifetime.len() - 1) as f32) as usize;
        let next_index = (index + 1).min(self.size_over_lifetime.len() - 1);

        let t = (life_ratio * (self.size_over_lifetime.len() - 1) as f32) - index as f32;
        let current = self.size_over_lifetime[index];
        let next = self.size_over_lifetime[next_index];

        (current + (next - current) * t) * self.initial_size
    }

    /// Evaluate color at given life ratio (0.0 to 1.0)
    pub fn evaluate_color(&self, life_ratio: f32) -> [f32; 4] {
        if self.color_over_lifetime.is_empty() {
            return self.initial_color;
        }

        let index = (life_ratio * (self.color_over_lifetime.len() - 1) as f32) as usize;
        let next_index = (index + 1).min(self.color_over_lifetime.len() - 1);

        let t = (life_ratio * (self.color_over_lifetime.len() - 1) as f32) - index as f32;
        let current = self.color_over_lifetime[index];
        let next = self.color_over_lifetime[next_index];

        [
            current[0] + (next[0] - current[0]) * t,
            current[1] + (next[1] - current[1]) * t,
            current[2] + (next[2] - current[2]) * t,
            current[3] + (next[3] - current[3]) * t,
        ]
    }
}
