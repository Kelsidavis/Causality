// Particle System - CPU-based particle simulation with instanced rendering

pub mod emitter;
pub mod particle;
pub mod system;

pub use emitter::{EmitterShape, ParticleEmitter};
pub use particle::{Particle, ParticleSettings};
pub use system::ParticleSystem;
