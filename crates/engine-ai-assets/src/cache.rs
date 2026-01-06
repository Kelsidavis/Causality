// Asset caching system for generated textures and skyboxes

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Metadata about a generated asset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetMetadata {
    /// Unique identifier for this asset
    pub id: String,
    /// Prompt used to generate the asset
    pub prompt: String,
    /// Negative prompt used
    pub negative_prompt: Option<String>,
    /// Image dimensions (width, height)
    pub dimensions: (u32, u32),
    /// Model that generated this asset
    pub model: String,
    /// Inference steps used
    pub steps: u32,
    /// Guidance scale used
    pub guidance_scale: f32,
    /// Random seed used (for reproducibility)
    pub seed: Option<u64>,
    /// Timestamp when generated (ISO 8601)
    pub generated_at: String,
    /// File path relative to cache directory
    pub file_path: String,
    /// Format of the image (png, jpg, etc.)
    pub format: String,
    /// Size in bytes
    pub file_size: u64,
}

impl AssetMetadata {
    /// Create metadata from generation parameters
    pub fn new(
        prompt: String,
        negative_prompt: Option<String>,
        dimensions: (u32, u32),
        model: String,
        steps: u32,
        guidance_scale: f32,
        seed: Option<u64>,
        file_path: String,
        format: String,
        file_size: u64,
    ) -> Self {
        let id = Self::generate_id(&prompt, &dimensions, seed);

        let generated_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs().to_string())
            .unwrap_or_else(|_| "0".to_string());

        Self {
            id,
            prompt,
            negative_prompt,
            dimensions,
            model,
            steps,
            guidance_scale,
            seed,
            generated_at,
            file_path,
            format,
            file_size,
        }
    }

    /// Generate a deterministic ID from prompt and parameters
    pub fn generate_id(prompt: &str, dimensions: &(u32, u32), seed: Option<u64>) -> String {
        let seed_str = seed.map(|s| s.to_string()).unwrap_or_default();
        let content = format!("{}_{}x{}_{}", prompt, dimensions.0, dimensions.1, seed_str);
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

/// Cache for storing and retrieving generated assets
pub struct AssetCache {
    cache_dir: PathBuf,
}

impl AssetCache {
    /// Create a new asset cache with the given directory
    pub fn new(cache_dir: impl AsRef<Path>) -> Result<Self> {
        let cache_dir = cache_dir.as_ref().to_path_buf();

        // Create cache directory structure
        fs::create_dir_all(&cache_dir)?;
        fs::create_dir_all(cache_dir.join("textures"))?;
        fs::create_dir_all(cache_dir.join("skyboxes"))?;
        fs::create_dir_all(cache_dir.join("metadata"))?;

        log::info!("Asset cache initialized at: {}", cache_dir.display());

        Ok(Self { cache_dir })
    }

    /// Store a generated asset in the cache
    pub fn store_asset(
        &self,
        asset_type: &str, // "texture" or "skybox"
        image_bytes: &[u8],
        metadata: &AssetMetadata,
    ) -> Result<String> {
        let asset_dir = self.cache_dir.join(asset_type);
        fs::create_dir_all(&asset_dir)?;

        let file_path = asset_dir.join(&metadata.file_path);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Write image file
        fs::write(&file_path, image_bytes)?;
        log::info!("Stored asset: {} ({} bytes)", metadata.id, image_bytes.len());

        // Write metadata file
        let metadata_path = self.cache_dir.join("metadata").join(format!("{}.json", metadata.id));
        let metadata_json = serde_json::to_string_pretty(metadata)?;
        fs::write(&metadata_path, metadata_json)?;

        Ok(metadata.id.clone())
    }

    /// Retrieve a cached asset by ID
    pub fn get_asset(&self, asset_id: &str) -> Result<(Vec<u8>, AssetMetadata)> {
        let metadata_path = self.cache_dir.join("metadata").join(format!("{}.json", asset_id));

        if !metadata_path.exists() {
            return Err(anyhow!("Asset not found in cache: {}", asset_id));
        }

        let metadata_json = fs::read_to_string(&metadata_path)?;
        let metadata: AssetMetadata = serde_json::from_str(&metadata_json)?;

        let file_path = self.cache_dir.join(&metadata.file_path);
        if !file_path.exists() {
            return Err(anyhow!("Asset file not found: {}", file_path.display()));
        }

        let image_bytes = fs::read(&file_path)?;

        Ok((image_bytes, metadata))
    }

    /// Check if an asset exists in cache
    pub fn has_asset(&self, asset_id: &str) -> bool {
        let metadata_path = self.cache_dir.join("metadata").join(format!("{}.json", asset_id));
        metadata_path.exists()
    }

    /// List all cached assets of a given type
    pub fn list_assets(&self, asset_type: &str) -> Result<Vec<AssetMetadata>> {
        let metadata_dir = self.cache_dir.join("metadata");
        let mut assets = Vec::new();

        if !metadata_dir.exists() {
            return Ok(assets);
        }

        for entry in fs::read_dir(metadata_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().map_or(false, |ext| ext == "json") {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(metadata) = serde_json::from_str::<AssetMetadata>(&content) {
                        if metadata.file_path.starts_with(asset_type) {
                            assets.push(metadata);
                        }
                    }
                }
            }
        }

        Ok(assets)
    }

    /// Clear all cached assets
    pub fn clear_all(&self) -> Result<()> {
        fs::remove_dir_all(&self.cache_dir)?;
        fs::create_dir_all(&self.cache_dir)?;
        fs::create_dir_all(self.cache_dir.join("textures"))?;
        fs::create_dir_all(self.cache_dir.join("skyboxes"))?;
        fs::create_dir_all(self.cache_dir.join("metadata"))?;

        log::info!("Asset cache cleared");
        Ok(())
    }

    /// Get cache statistics
    pub fn stats(&self) -> Result<CacheStats> {
        let metadata_dir = self.cache_dir.join("metadata");
        let mut total_assets = 0;
        let mut total_size = 0u64;
        let mut textures = 0;
        let mut skyboxes = 0;

        if metadata_dir.exists() {
            for entry in fs::read_dir(metadata_dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.extension().map_or(false, |ext| ext == "json") {
                    total_assets += 1;

                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok(metadata) = serde_json::from_str::<AssetMetadata>(&content) {
                            total_size += metadata.file_size;

                            if metadata.file_path.starts_with("textures") {
                                textures += 1;
                            } else if metadata.file_path.starts_with("skyboxes") {
                                skyboxes += 1;
                            }
                        }
                    }
                }
            }
        }

        Ok(CacheStats {
            total_assets,
            total_size,
            textures,
            skyboxes,
        })
    }
}

/// Statistics about the asset cache
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_assets: usize,
    pub total_size: u64,
    pub textures: usize,
    pub skyboxes: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_metadata_id_generation() {
        let id1 = AssetMetadata::generate_id("test prompt", &(512, 512), None);
        let id2 = AssetMetadata::generate_id("test prompt", &(512, 512), None);

        // Same parameters should generate same ID
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_metadata_id_differs_on_seed() {
        let id1 = AssetMetadata::generate_id("test prompt", &(512, 512), Some(42));
        let id2 = AssetMetadata::generate_id("test prompt", &(512, 512), Some(43));

        // Different seeds should generate different IDs
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_cache_creation() {
        let temp_dir = TempDir::new().unwrap();
        let cache = AssetCache::new(temp_dir.path()).unwrap();

        assert!(temp_dir.path().join("textures").exists());
        assert!(temp_dir.path().join("skyboxes").exists());
        assert!(temp_dir.path().join("metadata").exists());
    }

    #[test]
    fn test_store_and_retrieve_asset() {
        let temp_dir = TempDir::new().unwrap();
        let cache = AssetCache::new(temp_dir.path()).unwrap();

        let metadata = AssetMetadata::new(
            "test prompt".to_string(),
            None,
            (512, 512),
            "test-model".to_string(),
            50,
            7.5,
            Some(42),
            "test_image.png".to_string(),
            "png".to_string(),
            1024,
        );

        let test_data = b"fake image data";
        let asset_id = cache.store_asset("textures", test_data, &metadata).unwrap();

        assert!(cache.has_asset(&asset_id));

        let (retrieved, meta) = cache.get_asset(&asset_id).unwrap();
        assert_eq!(retrieved, test_data);
        assert_eq!(meta.id, metadata.id);
    }
}
