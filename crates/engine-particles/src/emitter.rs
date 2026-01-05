// Particle emitter - spawns particles based on shape and settings

use crate::particle::{Particle, ParticleSettings};
use glam::{Vec3, Vec4};
use rand::Rng;

/// Emitter shape types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EmitterShape {
    /// Emit from a single point
    Point,
    /// Emit from within a sphere
    Sphere { radius: f32 },
    /// Emit from within a cone
    Cone { angle: f32, radius: f32 },
    /// Emit from within a box
    Box { size: Vec3 },
}

/// Particle emitter
pub struct ParticleEmitter {
    /// Emitter position in world space
    pub position: Vec3,
    /// Emitter direction (for cone)
    pub direction: Vec3,
    /// Emitter shape
    pub shape: EmitterShape,
    /// Particle settings
    pub settings: ParticleSettings,
    /// Emission rate (particles per second)
    pub emission_rate: f32,
    /// Whether the emitter is active
    pub active: bool,
    /// Accumulated emission time
    emission_accumulator: f32,
}

impl ParticleEmitter {
    /// Create a new emitter
    pub fn new(position: Vec3, shape: EmitterShape, settings: ParticleSettings) -> Self {
        Self {
            position,
            direction: Vec3::Y,
            shape,
            settings,
            emission_rate: 10.0,
            active: true,
            emission_accumulator: 0.0,
        }
    }

    /// Set emission rate
    pub fn with_rate(mut self, rate: f32) -> Self {
        self.emission_rate = rate;
        self
    }

    /// Set direction (for cone emitters)
    pub fn with_direction(mut self, direction: Vec3) -> Self {
        self.direction = direction.normalize();
        self
    }

    /// Update emitter and spawn particles
    pub fn update(&mut self, delta_time: f32) -> Vec<Particle> {
        if !self.active {
            return Vec::new();
        }

        let mut particles = Vec::new();
        self.emission_accumulator += delta_time * self.emission_rate;

        while self.emission_accumulator >= 1.0 {
            particles.push(self.spawn_particle());
            self.emission_accumulator -= 1.0;
        }

        particles
    }

    /// Spawn a single particle
    fn spawn_particle(&self) -> Particle {
        let mut rng = rand::thread_rng();

        // Random position based on shape
        let offset = match self.shape {
            EmitterShape::Point => Vec3::ZERO,
            EmitterShape::Sphere { radius } => {
                let theta = rng.gen_range(0.0..std::f32::consts::TAU);
                let phi = rng.gen_range(0.0..std::f32::consts::PI);
                let r = rng.gen_range(0.0..radius);

                Vec3::new(
                    r * phi.sin() * theta.cos(),
                    r * phi.sin() * theta.sin(),
                    r * phi.cos(),
                )
            }
            EmitterShape::Cone { angle, radius } => {
                let theta = rng.gen_range(0.0..std::f32::consts::TAU);
                let phi = rng.gen_range(0.0..angle.to_radians());
                let r = rng.gen_range(0.0..radius);

                // Local cone space
                let local = Vec3::new(
                    r * phi.sin() * theta.cos(),
                    r * phi.sin() * theta.sin(),
                    r * phi.cos(),
                );

                // Rotate to align with direction
                // Simple rotation assuming direction is Y-up for now
                local
            }
            EmitterShape::Box { size } => Vec3::new(
                rng.gen_range(-size.x / 2.0..size.x / 2.0),
                rng.gen_range(-size.y / 2.0..size.y / 2.0),
                rng.gen_range(-size.z / 2.0..size.z / 2.0),
            ),
        };

        let position = self.position + offset;

        // Random velocity based on shape
        let velocity_direction = match self.shape {
            EmitterShape::Point => {
                // Random direction
                let theta = rng.gen_range(0.0..std::f32::consts::TAU);
                let phi = rng.gen_range(0.0..std::f32::consts::PI);
                Vec3::new(phi.sin() * theta.cos(), phi.sin() * theta.sin(), phi.cos())
            }
            EmitterShape::Sphere { .. } => {
                // Outward from center
                offset.normalize_or_zero()
            }
            EmitterShape::Cone { .. } => {
                // Along cone direction with some randomness
                self.direction
            }
            EmitterShape::Box { .. } => {
                // Upward
                Vec3::Y
            }
        };

        let speed = rng.gen_range(self.settings.speed_range.0..self.settings.speed_range.1);
        let velocity = velocity_direction * speed;

        // Random lifetime and size
        let lifetime = rng.gen_range(self.settings.lifetime_range.0..self.settings.lifetime_range.1);
        let size = rng.gen_range(self.settings.size_range.0..self.settings.size_range.1);

        // Initial color
        let color = if !self.settings.color_gradient.is_empty() {
            self.settings.color_gradient[0]
        } else {
            Vec4::ONE
        };

        Particle::new(position, velocity, color, size, lifetime)
    }

    /// Burst spawn particles
    pub fn burst(&mut self, count: usize) -> Vec<Particle> {
        (0..count).map(|_| self.spawn_particle()).collect()
    }
}

impl Default for EmitterShape {
    fn default() -> Self {
        EmitterShape::Point
    }
}
