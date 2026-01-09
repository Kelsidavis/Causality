// Common components for entities

use crate::entity::Component;
use crate::impl_component;
use serde::{Deserialize, Serialize};
use std::any::Any;

/// Mesh renderer component - references a mesh and material
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshRenderer {
    pub mesh_path: String,
    pub material_path: Option<String>,
}

impl MeshRenderer {
    pub fn new(mesh_path: String) -> Self {
        Self {
            mesh_path,
            material_path: None,
        }
    }

    pub fn with_material(mut self, material_path: String) -> Self {
        self.material_path = Some(material_path);
        self
    }
}

impl_component!(MeshRenderer);

/// Camera component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Camera {
    pub fov: f32,
    pub near: f32,
    pub far: f32,
    pub is_active: bool,
}

impl Camera {
    pub fn new(fov: f32, near: f32, far: f32) -> Self {
        Self {
            fov,
            near,
            far,
            is_active: true,
        }
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new(45.0_f32.to_radians(), 0.1, 100.0)
    }
}

impl_component!(Camera);

/// Light component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LightType {
    Directional { direction: [f32; 3] },
    Point { range: f32, intensity: f32 },
    Spot { direction: [f32; 3], angle: f32, range: f32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Light {
    pub light_type: LightType,
    pub color: [f32; 3],
    pub intensity: f32,
}

impl Light {
    pub fn directional(direction: [f32; 3], color: [f32; 3], intensity: f32) -> Self {
        Self {
            light_type: LightType::Directional { direction },
            color,
            intensity,
        }
    }

    pub fn point(color: [f32; 3], intensity: f32, range: f32) -> Self {
        Self {
            light_type: LightType::Point { range, intensity },
            color,
            intensity,
        }
    }
}

impl_component!(Light);

/// Audio source component - plays sound at entity position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSource {
    pub audio_path: String,
    pub volume: f32,
    pub max_distance: f32,
    pub playing: bool,
    pub looping: bool,
    pub play_on_start: bool,
}

impl AudioSource {
    pub fn new(audio_path: String) -> Self {
        Self {
            audio_path,
            volume: 1.0,
            max_distance: 50.0,
            playing: false,
            looping: false,
            play_on_start: false,
        }
    }

    pub fn with_volume(mut self, volume: f32) -> Self {
        self.volume = volume;
        self
    }

    pub fn with_looping(mut self, looping: bool) -> Self {
        self.looping = looping;
        self
    }

    pub fn with_play_on_start(mut self, play_on_start: bool) -> Self {
        self.play_on_start = play_on_start;
        self
    }
}

impl_component!(AudioSource);

/// Audio listener component - typically attached to camera
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioListener {
    pub active: bool,
}

impl AudioListener {
    pub fn new() -> Self {
        Self { active: true }
    }
}

impl Default for AudioListener {
    fn default() -> Self {
        Self::new()
    }
}

impl_component!(AudioListener);

/// Particle emitter component - emits particles with configurable properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticleEmitter {
    pub enabled: bool,
    pub max_particles: u32,
    pub shape: String,              // Serialized EmitterShape
    pub rate: f32,
    pub initial_velocity: [f32; 3],
    pub velocity_randomness: f32,
    pub lifetime: f32,
    pub lifetime_randomness: f32,
    pub initial_size: f32,
    pub initial_color: [f32; 4],
    pub gravity: [f32; 3],
    pub texture_path: Option<String>,
    pub blend_mode: BlendMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BlendMode {
    Alpha,      // Standard alpha blending
    Additive,   // Additive (fire, sparks)
    Multiply,   // Multiplicative (smoke)
}

impl ParticleEmitter {
    pub fn new() -> Self {
        Self {
            enabled: true,
            max_particles: 1000,
            shape: "Point".to_string(),
            rate: 10.0,
            initial_velocity: [0.0, 1.0, 0.0],
            velocity_randomness: 0.1,
            lifetime: 1.0,
            lifetime_randomness: 0.0,
            initial_size: 1.0,
            initial_color: [1.0, 1.0, 1.0, 1.0],
            gravity: [0.0, -9.81, 0.0],
            texture_path: None,
            blend_mode: BlendMode::Alpha,
        }
    }
}

impl Default for ParticleEmitter {
    fn default() -> Self {
        Self::new()
    }
}

impl_component!(ParticleEmitter);
