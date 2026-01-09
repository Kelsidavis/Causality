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

/// Water plane component - renders animated water with waves and fresnel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Water {
    pub mesh_path: String,
    pub texture_path: Option<String>,
    pub wave_speed: f32,
    pub wave_frequency: f32,
    pub wave_amplitude: f32,
    pub color: [f32; 3],
    pub transparency: f32,
    /// Flow direction (normalized XZ)
    pub flow_direction: [f32; 2],
    /// Flow speed in units per second
    pub flow_speed: f32,
}

impl Water {
    pub fn new(mesh_path: String) -> Self {
        Self {
            mesh_path,
            texture_path: Some("textures/water.png".to_string()),
            wave_speed: 0.5,
            wave_frequency: 2.0,
            wave_amplitude: 0.1,
            color: [0.2, 0.5, 0.8], // Blue-ish water
            transparency: 0.6,
            flow_direction: [1.0, 0.0], // Flow in +X direction
            flow_speed: 0.0, // No flow by default
        }
    }

    pub fn with_texture(mut self, texture_path: String) -> Self {
        self.texture_path = Some(texture_path);
        self
    }
}

impl Default for Water {
    fn default() -> Self {
        Self::new("water_cube".to_string())
    }
}

impl_component!(Water);

/// A computed water body from terrain flood-fill (runtime only, not serialized)
#[derive(Debug, Clone)]
pub struct WaterBody {
    pub id: u32,
    pub surface_level: f32,
    pub bounds_min: [f32; 3],
    pub bounds_max: [f32; 3],
    pub mesh_name: String,
    pub connected_to: Vec<u32>,
    pub flow_direction: Option<[f32; 2]>,
    pub flow_speed: f32,
}

/// Terrain-aware water that fills depressions via flood-fill
/// Water is computed at scene load based on terrain heightmap
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerrainWater {
    /// Reference to terrain entity name (must have terrain heightmap)
    pub terrain_reference: String,
    /// Global ground water level - water fills below this height
    pub ground_water_level: f32,
    /// Minimum water depth to create a water body (filters tiny puddles)
    pub min_water_depth: f32,
    /// Minimum area in grid cells to create a water body
    pub min_water_area: usize,
    /// Water rendering properties
    pub wave_speed: f32,
    pub wave_amplitude: f32,
    pub color: [f32; 3],
    pub transparency: f32,
    pub texture_path: Option<String>,
    /// Computed water bodies (populated at load time, not serialized)
    #[serde(skip)]
    pub water_bodies: Vec<WaterBody>,
}

impl TerrainWater {
    pub fn new(terrain_reference: String, ground_water_level: f32) -> Self {
        Self {
            terrain_reference,
            ground_water_level,
            min_water_depth: 0.1,
            min_water_area: 4,
            wave_speed: 0.5,
            wave_amplitude: 0.05,
            color: [0.2, 0.5, 0.8],
            transparency: 0.6,
            texture_path: Some("water".to_string()),
            water_bodies: Vec::new(),
        }
    }
}

impl Default for TerrainWater {
    fn default() -> Self {
        Self::new("Terrain".to_string(), 2.0)
    }
}

impl_component!(TerrainWater);
