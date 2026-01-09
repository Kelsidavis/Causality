// Material data structure for PBR rendering

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Material {
    pub name: String,

    // PBR texture maps (optional paths)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub albedo_texture: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub normal_texture: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub metallic_roughness_texture: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub ao_texture: Option<String>,

    // PBR parameters (fallbacks when textures are missing)
    pub base_color: [f32; 4],      // RGBA
    pub metallic: f32,              // 0.0 = dielectric, 1.0 = metal
    pub roughness: f32,             // 0.0 = smooth, 1.0 = rough
    pub ao_factor: f32,             // Ambient occlusion multiplier

    // Emissive properties
    pub emissive_color: [f32; 3],   // RGB
    pub emissive_strength: f32,

    // Transparency and rendering
    pub alpha_mode: AlphaMode,
    pub alpha_cutoff: f32,          // Used for AlphaMode::Mask
    pub double_sided: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlphaMode {
    /// Fully opaque material
    Opaque,
    /// Alpha masking (binary transparency based on cutoff)
    Mask,
    /// Alpha blending (transparency)
    Blend,
}

impl Material {
    /// Create a new material with default PBR values
    pub fn new(name: String) -> Self {
        Self {
            name,
            albedo_texture: None,
            normal_texture: None,
            metallic_roughness_texture: None,
            ao_texture: None,
            base_color: [1.0, 1.0, 1.0, 1.0],
            metallic: 0.0,
            roughness: 0.5,
            ao_factor: 1.0,
            emissive_color: [0.0, 0.0, 0.0],
            emissive_strength: 0.0,
            alpha_mode: AlphaMode::Opaque,
            alpha_cutoff: 0.5,
            double_sided: false,
        }
    }

    /// Create a simple material with just an albedo texture
    pub fn with_albedo(name: String, albedo_path: String) -> Self {
        Self {
            albedo_texture: Some(albedo_path),
            ..Self::new(name)
        }
    }

    /// Create a full PBR material with all texture maps
    pub fn with_pbr_textures(
        name: String,
        albedo: String,
        normal: Option<String>,
        metallic_roughness: Option<String>,
        ao: Option<String>,
    ) -> Self {
        Self {
            albedo_texture: Some(albedo),
            normal_texture: normal,
            metallic_roughness_texture: metallic_roughness,
            ao_texture: ao,
            ..Self::new(name)
        }
    }

    /// Create a metallic material (like polished metal)
    pub fn metallic(name: String, base_color: [f32; 3]) -> Self {
        Self {
            base_color: [base_color[0], base_color[1], base_color[2], 1.0],
            metallic: 1.0,
            roughness: 0.2,
            ..Self::new(name)
        }
    }

    /// Create a dielectric material (like plastic or stone)
    pub fn dielectric(name: String, base_color: [f32; 3], roughness: f32) -> Self {
        Self {
            base_color: [base_color[0], base_color[1], base_color[2], 1.0],
            metallic: 0.0,
            roughness,
            ..Self::new(name)
        }
    }

    /// Create an emissive material (glowing)
    pub fn emissive(name: String, color: [f32; 3], strength: f32) -> Self {
        Self {
            base_color: [color[0], color[1], color[2], 1.0],
            emissive_color: color,
            emissive_strength: strength,
            ..Self::new(name)
        }
    }

    /// Set material to use alpha blending
    pub fn with_transparency(mut self, alpha: f32) -> Self {
        self.base_color[3] = alpha;
        self.alpha_mode = AlphaMode::Blend;
        self
    }

    /// Set material to use alpha masking
    pub fn with_alpha_mask(mut self, cutoff: f32) -> Self {
        self.alpha_mode = AlphaMode::Mask;
        self.alpha_cutoff = cutoff;
        self
    }
}

impl Default for Material {
    fn default() -> Self {
        Self::new("Default".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_material_creation() {
        let mat = Material::new("Test".to_string());
        assert_eq!(mat.name, "Test");
        assert_eq!(mat.base_color, [1.0, 1.0, 1.0, 1.0]);
        assert_eq!(mat.metallic, 0.0);
        assert_eq!(mat.roughness, 0.5);
    }

    #[test]
    fn test_metallic_material() {
        let mat = Material::metallic("Gold".to_string(), [1.0, 0.84, 0.0]);
        assert_eq!(mat.metallic, 1.0);
        assert_eq!(mat.roughness, 0.2);
        assert_eq!(mat.base_color[0], 1.0);
        assert_eq!(mat.base_color[1], 0.84);
    }

    #[test]
    fn test_transparency() {
        let mat = Material::new("Glass".to_string()).with_transparency(0.5);
        assert_eq!(mat.alpha_mode, AlphaMode::Blend);
        assert_eq!(mat.base_color[3], 0.5);
    }

    #[test]
    fn test_serialization() {
        let mat = Material::with_albedo(
            "TestMat".to_string(),
            "textures/test.png".to_string()
        );

        let yaml = serde_yaml::to_string(&mat).unwrap();
        let deserialized: Material = serde_yaml::from_str(&yaml).unwrap();

        assert_eq!(mat, deserialized);
    }
}
