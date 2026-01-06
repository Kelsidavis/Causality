// Asset generator - orchestrates API client and caching

use crate::api::ApiClient;
use crate::cache::{AssetCache, AssetMetadata};
use anyhow::Result;
use uuid::Uuid;

/// Request for texture generation
#[derive(Debug, Clone)]
pub struct TextureGenerationRequest {
    /// Text prompt for the image
    pub prompt: String,
    /// Optional negative prompt
    pub negative_prompt: Option<String>,
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
    /// Number of inference steps (more = better quality, slower)
    pub steps: u32,
    /// Guidance scale (how closely to follow prompt)
    pub guidance_scale: f32,
    /// Seed for reproducibility (None = random)
    pub seed: Option<u64>,
    /// Use cached result if available
    pub use_cache: bool,
}

impl Default for TextureGenerationRequest {
    fn default() -> Self {
        Self {
            prompt: String::new(),
            negative_prompt: None,
            width: 512,
            height: 512,
            steps: 50,
            guidance_scale: 7.5,
            seed: None,
            use_cache: true,
        }
    }
}

/// Generated asset with metadata
#[derive(Debug, Clone)]
pub struct GeneratedAsset {
    /// Image bytes (PNG format)
    pub image_data: Vec<u8>,
    /// Asset metadata
    pub metadata: AssetMetadata,
    /// Whether this was retrieved from cache
    pub from_cache: bool,
}

/// Main asset generator
pub struct AssetGenerator {
    api_client: Box<dyn ApiClient>,
    cache: AssetCache,
    model_name: String,
}

impl AssetGenerator {
    /// Create a new asset generator
    pub fn new(api_client: Box<dyn ApiClient>, cache: AssetCache) -> Result<Self> {
        let model_name = api_client.model_name().to_string();

        Ok(Self {
            api_client,
            cache,
            model_name,
        })
    }

    /// Generate a texture from a prompt
    pub async fn generate_texture(
        &self,
        request: &TextureGenerationRequest,
    ) -> Result<GeneratedAsset> {
        // Check cache first if enabled
        if request.use_cache {
            let asset_id = AssetMetadata::generate_id(
                &request.prompt,
                &(request.width, request.height),
                request.seed,
            );

            if self.cache.has_asset(&asset_id) {
                log::info!("Using cached asset: {}", asset_id);
                let (image_data, metadata) = self.cache.get_asset(&asset_id)?;
                return Ok(GeneratedAsset {
                    image_data,
                    metadata,
                    from_cache: true,
                });
            }
        }

        log::info!(
            "Generating texture: {} ({}x{})",
            request.prompt,
            request.width,
            request.height
        );

        // Call API to generate image
        let image_bytes = self
            .api_client
            .generate_image(
                &request.prompt,
                request.width,
                request.height,
                request.seed,
            )
            .await?;

        // Create metadata
        let file_name = format!("{}.png", Uuid::new_v4());
        let metadata = AssetMetadata::new(
            request.prompt.clone(),
            request.negative_prompt.clone(),
            (request.width, request.height),
            self.model_name.clone(),
            request.steps,
            request.guidance_scale,
            request.seed,
            format!("textures/{}", file_name),
            "png".to_string(),
            image_bytes.len() as u64,
        );

        // Store in cache
        self.cache
            .store_asset("textures", &image_bytes, &metadata)?;

        log::info!(
            "Successfully generated texture: {} ({} bytes)",
            metadata.id,
            image_bytes.len()
        );

        Ok(GeneratedAsset {
            image_data: image_bytes,
            metadata,
            from_cache: false,
        })
    }

    /// Generate a skybox from a prompt
    pub async fn generate_skybox(
        &self,
        prompt: &str,
        seed: Option<u64>,
    ) -> Result<GeneratedAsset> {
        let request = TextureGenerationRequest {
            prompt: format!("360 degree panoramic skybox: {}", prompt),
            negative_prompt: Some("distorted, warped, artifacts, watermark".to_string()),
            width: 2048,
            height: 1024,
            steps: 50,
            guidance_scale: 7.5,
            seed,
            use_cache: true,
        };

        self.generate_texture(&request).await
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> Result<CacheStatistics> {
        let stats = self.cache.stats()?;

        Ok(CacheStatistics {
            total_assets: stats.total_assets,
            total_size: stats.total_size,
            textures: stats.textures,
            skyboxes: stats.skyboxes,
        })
    }

    /// Clear cache
    pub fn clear_cache(&self) -> Result<()> {
        self.cache.clear_all()
    }
}

/// Statistics about generated assets
#[derive(Debug, Clone)]
pub struct CacheStatistics {
    pub total_assets: usize,
    pub total_size: u64,
    pub textures: usize,
    pub skyboxes: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_texture_request_default() {
        let req = TextureGenerationRequest::default();
        assert_eq!(req.width, 512);
        assert_eq!(req.height, 512);
        assert_eq!(req.steps, 50);
        assert_eq!(req.guidance_scale, 7.5);
        assert!(req.use_cache);
    }

    #[test]
    fn test_skybox_prompt_formatting() {
        let prompt = "sunset";
        let skybox_prompt = format!("360 degree panoramic skybox: {}", prompt);
        assert!(skybox_prompt.contains("skybox"));
        assert!(skybox_prompt.contains(prompt));
    }
}
