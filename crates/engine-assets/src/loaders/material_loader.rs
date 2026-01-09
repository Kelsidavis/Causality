// Material loader - supports YAML and JSON formats

use crate::material::Material;
use anyhow::{Context, Result};
use std::path::Path;

/// Load a material from a YAML or JSON file
pub fn load_material<P: AsRef<Path>>(path: P) -> Result<Material> {
    let path_ref = path.as_ref();
    let content = std::fs::read_to_string(path_ref)
        .with_context(|| format!("Failed to read material file: {:?}", path_ref))?;

    // Determine format by file extension
    let extension = path_ref
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    let material: Material = match extension.to_lowercase().as_str() {
        "yaml" | "yml" => serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse YAML material: {:?}", path_ref))?,
        "json" => serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse JSON material: {:?}", path_ref))?,
        _ => {
            // Try YAML first, then JSON
            serde_yaml::from_str(&content)
                .or_else(|_| serde_json::from_str(&content))
                .with_context(|| {
                    format!(
                        "Failed to parse material file (tried YAML and JSON): {:?}",
                        path_ref
                    )
                })?
        }
    };

    Ok(material)
}

/// Save a material to a YAML or JSON file
pub fn save_material<P: AsRef<Path>>(material: &Material, path: P) -> Result<()> {
    let path_ref = path.as_ref();

    // Determine format by file extension
    let extension = path_ref
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    let content = match extension.to_lowercase().as_str() {
        "yaml" | "yml" => serde_yaml::to_string(material)
            .with_context(|| format!("Failed to serialize material to YAML: {:?}", path_ref))?,
        "json" => serde_json::to_string_pretty(material)
            .with_context(|| format!("Failed to serialize material to JSON: {:?}", path_ref))?,
        _ => {
            // Default to YAML
            serde_yaml::to_string(material)
                .with_context(|| format!("Failed to serialize material to YAML: {:?}", path_ref))?
        }
    };

    std::fs::write(path_ref, content)
        .with_context(|| format!("Failed to write material file: {:?}", path_ref))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::material::AlphaMode;
    use tempfile::tempdir;

    #[test]
    fn test_yaml_roundtrip() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_material.yaml");

        let material = Material::with_pbr_textures(
            "TestMaterial".to_string(),
            "textures/albedo.png".to_string(),
            Some("textures/normal.png".to_string()),
            Some("textures/metallic_roughness.png".to_string()),
            None,
        );

        // Save
        save_material(&material, &file_path).unwrap();

        // Load
        let loaded = load_material(&file_path).unwrap();

        assert_eq!(loaded.name, material.name);
        assert_eq!(loaded.albedo_texture, material.albedo_texture);
        assert_eq!(loaded.normal_texture, material.normal_texture);
    }

    #[test]
    fn test_json_roundtrip() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_material.json");

        let material = Material::metallic("Gold".to_string(), [1.0, 0.84, 0.0]);

        // Save
        save_material(&material, &file_path).unwrap();

        // Load
        let loaded = load_material(&file_path).unwrap();

        assert_eq!(loaded.name, material.name);
        assert_eq!(loaded.metallic, 1.0);
        assert_eq!(loaded.roughness, 0.2);
    }

    #[test]
    fn test_load_nonexistent_file() {
        let result = load_material("nonexistent.yaml");
        assert!(result.is_err());
    }
}
