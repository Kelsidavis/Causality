// Particle system - manages all particle emitters and active particles

use crate::emitter::ParticleEmitter;
use crate::particle::{Particle, ParticleSettings};
use glam::Vec3;

/// Particle system manager
pub struct ParticleSystem {
    /// All active particles
    particles: Vec<Particle>,
    /// All registered emitters
    emitters: Vec<ParticleEmitter>,
    /// Maximum number of particles
    max_particles: usize,
}

impl ParticleSystem {
    /// Create a new particle system
    pub fn new(max_particles: usize) -> Self {
        Self {
            particles: Vec::with_capacity(max_particles),
            emitters: Vec::new(),
            max_particles,
        }
    }

    /// Add an emitter
    pub fn add_emitter(&mut self, emitter: ParticleEmitter) {
        self.emitters.push(emitter);
    }

    /// Remove an emitter by index
    pub fn remove_emitter(&mut self, index: usize) {
        if index < self.emitters.len() {
            self.emitters.remove(index);
        }
    }

    /// Get number of active particles
    pub fn particle_count(&self) -> usize {
        self.particles.len()
    }

    /// Get number of emitters
    pub fn emitter_count(&self) -> usize {
        self.emitters.len()
    }

    /// Get all active particles
    pub fn particles(&self) -> &[Particle] {
        &self.particles
    }

    /// Get mutable access to emitters
    pub fn emitters_mut(&mut self) -> &mut [ParticleEmitter] {
        &mut self.emitters
    }

    /// Update all particles and emitters
    pub fn update(&mut self, delta_time: f32) {
        // Update existing particles
        self.particles.retain_mut(|particle| {
            let alive = particle.update(delta_time, particle.velocity);

            // Update color and size based on age
            if alive {
                let age = particle.normalized_age();
                // Color would be updated here if we had settings
                // For now, fade alpha
                particle.color.w = 1.0 - age;
            }

            alive
        });

        // Update emitters and spawn new particles
        for emitter in &mut self.emitters {
            let new_particles = emitter.update(delta_time);

            for particle in new_particles {
                if self.particles.len() < self.max_particles {
                    self.particles.push(particle);
                } else {
                    break;
                }
            }
        }
    }

    /// Clear all particles
    pub fn clear_particles(&mut self) {
        self.particles.clear();
    }

    /// Clear all emitters
    pub fn clear_emitters(&mut self) {
        self.emitters.clear();
    }

    /// Clear everything
    pub fn clear(&mut self) {
        self.particles.clear();
        self.emitters.clear();
    }
}

impl Default for ParticleSystem {
    fn default() -> Self {
        Self::new(10000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::emitter::EmitterShape;

    #[test]
    fn test_particle_system_basic() {
        let mut system = ParticleSystem::new(100);

        let settings = ParticleSettings::default();
        let emitter = ParticleEmitter::new(Vec3::ZERO, EmitterShape::Point, settings);

        system.add_emitter(emitter);
        assert_eq!(system.emitter_count(), 1);

        // Update for 1 second
        system.update(1.0);

        // Should have spawned some particles
        assert!(system.particle_count() > 0);
    }

    #[test]
    fn test_max_particles() {
        let mut system = ParticleSystem::new(10);

        let settings = ParticleSettings {
            emission_rate: 100.0,
            ..Default::default()
        };

        let mut emitter = ParticleEmitter::new(Vec3::ZERO, EmitterShape::Point, settings);
        emitter.emission_rate = 100.0;

        system.add_emitter(emitter);

        // Update for 1 second - should cap at 10 particles
        system.update(1.0);

        assert!(system.particle_count() <= 10);
    }
}
