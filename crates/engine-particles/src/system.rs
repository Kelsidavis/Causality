// Particle system management

use crate::emitter::EmitterProperties;
use crate::particle::GpuParticle;
use glam::Vec3;
use std::collections::VecDeque;

/// Particle system managing a pool of particles
pub struct ParticleSystem {
    /// Maximum number of particles
    pub max_particles: u32,

    /// Currently active particles
    pub particles: Vec<GpuParticle>,

    /// Free particle indices for reuse
    free_indices: VecDeque<usize>,

    /// Accumulator for spawning (handles fractional rates)
    spawn_accumulator: f32,

    /// Emitter properties
    pub properties: EmitterProperties,

    /// World position of emitter
    pub position: Vec3,

    /// Is emitter enabled
    pub enabled: bool,
}

impl ParticleSystem {
    /// Create a new particle system
    pub fn new(max_particles: u32, properties: EmitterProperties) -> Self {
        // Initialize all particles as dead (position.y = -9999.0)
        let mut particles = vec![GpuParticle::default(); max_particles as usize];
        for particle in &mut particles {
            particle.kill(); // Mark as dead
        }
        let free_indices: VecDeque<usize> = (0..max_particles as usize).collect();

        Self {
            max_particles,
            particles,
            free_indices,
            spawn_accumulator: 0.0,
            properties,
            position: Vec3::ZERO,
            enabled: true,
        }
    }

    /// Update particle system (spawn new particles)
    pub fn update(&mut self, delta_time: f32) {
        if !self.enabled {
            return;
        }

        // Calculate how many particles to spawn
        self.spawn_accumulator += self.properties.rate * delta_time;

        let particles_to_spawn = self.spawn_accumulator.floor() as u32;
        self.spawn_accumulator -= particles_to_spawn as f32;

        // Spawn particles
        for _ in 0..particles_to_spawn {
            self.spawn_particle();
        }

        // Debug logging
        if particles_to_spawn > 0 {
            let alive = self.particles.iter().filter(|p| p.is_alive()).count();
            log::info!("Spawned {} particles, {} alive total, {} free slots",
                      particles_to_spawn, alive, self.free_indices.len());
        }
    }

    /// Spawn a single particle
    fn spawn_particle(&mut self) {
        // Get a free particle index
        let Some(index) = self.free_indices.pop_front() else {
            // No free particles, pool is full
            return;
        };

        // Sample position within emitter shape
        let local_pos = self.properties.shape.sample_position();
        let world_pos = self.position + local_pos;

        // Sample velocity
        let velocity = self.properties.sample_velocity();

        // Sample lifetime
        let lifetime = self.properties.sample_lifetime();

        // Create particle
        let particle = GpuParticle::new(
            world_pos,
            velocity,
            self.properties.initial_color,
            self.properties.initial_size,
            lifetime,
        );

        log::debug!("Spawned particle at {:?}, size: {}, color: {:?}, velocity: {:?}",
                   world_pos, self.properties.initial_size, self.properties.initial_color, velocity);

        self.particles[index] = particle;
    }

    /// Kill dead particles and return them to the pool
    pub fn collect_dead_particles(&mut self) {
        for (i, particle) in self.particles.iter_mut().enumerate() {
            if !particle.is_alive() && particle.lifetime > 0.0 {
                // Particle just died, return to pool
                particle.kill();
                self.free_indices.push_back(i);
            }
        }
    }

    /// Get number of active particles
    pub fn active_particle_count(&self) -> usize {
        self.max_particles as usize - self.free_indices.len()
    }

    /// Get particles as slice for GPU upload
    pub fn particles_slice(&self) -> &[GpuParticle] {
        &self.particles
    }
}

impl Default for ParticleSystem {
    fn default() -> Self {
        Self::new(1000, EmitterProperties::default())
    }
}
