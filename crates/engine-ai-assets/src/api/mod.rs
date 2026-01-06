// API clients for different Stable Diffusion services

pub mod huggingface;

pub use huggingface::HuggingFaceClient;

use serde::{Deserialize, Serialize};

/// Trait for API clients that can generate images
#[async_trait::async_trait]
pub trait ApiClient: Send + Sync {
    /// Generate an image from a text prompt
    async fn generate_image(
        &self,
        prompt: &str,
        width: u32,
        height: u32,
        seed: Option<u64>,
    ) -> anyhow::Result<Vec<u8>>;

    /// Get model info
    fn model_name(&self) -> &str;

    /// Check if API is available
    async fn health_check(&self) -> anyhow::Result<()>;
}

/// Request parameters for image generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationRequest {
    pub prompt: String,
    pub negative_prompt: Option<String>,
    pub width: u32,
    pub height: u32,
    pub num_inference_steps: Option<u32>,
    pub guidance_scale: Option<f32>,
    pub seed: Option<u64>,
}

impl Default for GenerationRequest {
    fn default() -> Self {
        Self {
            prompt: String::new(),
            negative_prompt: Some("blurry, low quality, distorted".to_string()),
            width: 512,
            height: 512,
            num_inference_steps: Some(50),
            guidance_scale: Some(7.5),
            seed: None,
        }
    }
}

/// Response from image generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationResponse {
    pub image: Vec<u8>,
    pub seed: u64,
    pub steps: u32,
}
