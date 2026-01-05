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
