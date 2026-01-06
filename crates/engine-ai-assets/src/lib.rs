// Engine AI Assets - AI-powered texture and asset generation
//
// Provides tools for generating textures, skyboxes, and other assets
// using Stable Diffusion through various APIs (Hugging Face, Replicate, local)

pub mod api;
pub mod cache;
pub mod generator;
pub mod models;
pub mod prompt;

pub use generator::{AssetGenerator, TextureGenerationRequest, GeneratedAsset};
pub use cache::{AssetCache, AssetMetadata};
pub use api::{ApiClient, LocalClient, HuggingFaceClient};
pub use models::AiModel;
pub use prompt::{PromptOptimizer, QualityLevel};

/// Configuration for AI asset generation
#[derive(Debug, Clone)]
pub struct AiAssetConfig {
    /// API key for authentication
    pub api_key: String,
    /// Model to use (e.g., "stabilityai/stable-diffusion-2-1")
    pub model: String,
    /// Cache directory for generated assets
    pub cache_dir: String,
    /// API timeout in seconds
    pub timeout_seconds: u64,
    /// Quality level: "best", "high", "standard", "fast"
    pub quality: String,
    /// Enable upscaling to 4K
    pub upscaler_enabled: bool,
}

impl Default for AiAssetConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            model: "stabilityai/stable-diffusion-2-1".to_string(),
            cache_dir: "./generated_assets".to_string(),
            timeout_seconds: 300,
            quality: "high".to_string(),
            upscaler_enabled: true,
        }
    }
}

impl AiAssetConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> anyhow::Result<Self> {
        let api_key = std::env::var("HF_API_KEY")
            .or_else(|_| std::env::var("REPLICATE_API_TOKEN"))
            .map_err(|_| anyhow::anyhow!("No API key found. Set HF_API_KEY or REPLICATE_API_TOKEN"))?;

        Ok(Self {
            api_key,
            model: std::env::var("HF_MODEL").unwrap_or_else(|_| "stabilityai/stable-diffusion-2-1".to_string()),
            cache_dir: std::env::var("TEXTURE_CACHE_DIR").unwrap_or_else(|_| "./generated_assets".to_string()),
            timeout_seconds: std::env::var("HF_TIMEOUT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(300),
            quality: std::env::var("TEXTURE_QUALITY").unwrap_or_else(|_| "high".to_string()),
            upscaler_enabled: std::env::var("UPSCALER_ENABLED")
                .ok()
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(true),
        })
    }
}
