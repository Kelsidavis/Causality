// Supported AI models and their configurations

use serde::{Deserialize, Serialize};

/// Available AI models for asset generation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AiModel {
    /// Stable Diffusion 2.1 via Hugging Face
    StableDiffusion21,
    /// Stable Diffusion 3 via Hugging Face
    StableDiffusion3,
    /// SDXL (Stable Diffusion XL) - better quality
    StableDiffusionXL,
}

impl AiModel {
    /// Get the model identifier for the API
    pub fn model_id(&self) -> &'static str {
        match self {
            Self::StableDiffusion21 => "stabilityai/stable-diffusion-2-1",
            Self::StableDiffusion3 => "stabilityai/stable-diffusion-3",
            Self::StableDiffusionXL => "stabilityai/stable-diffusion-xl",
        }
    }

    /// Get human-readable name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::StableDiffusion21 => "Stable Diffusion 2.1",
            Self::StableDiffusion3 => "Stable Diffusion 3",
            Self::StableDiffusionXL => "Stable Diffusion XL",
        }
    }

    /// Get recommended inference steps for this model
    pub fn recommended_steps(&self) -> u32 {
        match self {
            Self::StableDiffusion21 => 50,
            Self::StableDiffusion3 => 40,
            Self::StableDiffusionXL => 30,
        }
    }

    /// Get recommended guidance scale
    pub fn recommended_guidance(&self) -> f32 {
        match self {
            Self::StableDiffusion21 => 7.5,
            Self::StableDiffusion3 => 7.5,
            Self::StableDiffusionXL => 7.0,
        }
    }

    /// Check if this model supports a given resolution
    pub fn supports_resolution(&self, width: u32, height: u32) -> bool {
        // All models support 256-2048 in multiples of 64
        (width >= 256 && width <= 2048 && width % 64 == 0)
            && (height >= 256 && height <= 2048 && height % 64 == 0)
    }

    /// Get estimated API cost per request (in credits or USD equivalent)
    pub fn estimated_cost(&self) -> f32 {
        // Approximate costs based on inference time
        match self {
            Self::StableDiffusion21 => 0.01,
            Self::StableDiffusion3 => 0.015,
            Self::StableDiffusionXL => 0.02,
        }
    }

    /// Get all available models
    pub fn all_models() -> &'static [AiModel] {
        &[
            Self::StableDiffusion21,
            Self::StableDiffusion3,
            Self::StableDiffusionXL,
        ]
    }
}

impl Default for AiModel {
    fn default() -> Self {
        Self::StableDiffusion21
    }
}

impl std::fmt::Display for AiModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Model capabilities
#[derive(Debug, Clone)]
pub struct ModelCapabilities {
    pub model: AiModel,
    pub supports_negative_prompt: bool,
    pub supports_custom_steps: bool,
    pub supports_seed: bool,
    pub supports_custom_guidance: bool,
    pub max_prompt_length: usize,
    pub supported_formats: Vec<String>,
}

impl ModelCapabilities {
    /// Get capabilities for a model
    pub fn for_model(model: AiModel) -> Self {
        Self {
            model,
            supports_negative_prompt: true,
            supports_custom_steps: true,
            supports_seed: true,
            supports_custom_guidance: true,
            max_prompt_length: 1000,
            supported_formats: vec!["png".to_string(), "jpg".to_string()],
        }
    }

    /// Validate a prompt length
    pub fn validate_prompt(&self, prompt: &str) -> Result<(), String> {
        if prompt.len() > self.max_prompt_length {
            Err(format!(
                "Prompt exceeds maximum length of {}",
                self.max_prompt_length
            ))
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_ids() {
        assert_eq!(
            AiModel::StableDiffusion21.model_id(),
            "stabilityai/stable-diffusion-2-1"
        );
        assert_eq!(
            AiModel::StableDiffusionXL.model_id(),
            "stabilityai/stable-diffusion-xl"
        );
    }

    #[test]
    fn test_recommended_parameters() {
        let model = AiModel::StableDiffusionXL;
        assert_eq!(model.recommended_steps(), 30);
        assert_eq!(model.recommended_guidance(), 7.0);
    }

    #[test]
    fn test_resolution_support() {
        let model = AiModel::StableDiffusion21;

        // Valid resolutions
        assert!(model.supports_resolution(512, 512));
        assert!(model.supports_resolution(768, 512));
        assert!(model.supports_resolution(1024, 1024));

        // Invalid resolutions
        assert!(!model.supports_resolution(511, 512)); // Too small
        assert!(!model.supports_resolution(2049, 512)); // Too large
        assert!(!model.supports_resolution(513, 512)); // Not multiple of 64
    }

    #[test]
    fn test_all_models() {
        let models = AiModel::all_models();
        assert!(models.len() >= 3);
        assert!(models.contains(&AiModel::StableDiffusion21));
        assert!(models.contains(&AiModel::StableDiffusionXL));
    }

    #[test]
    fn test_capabilities() {
        let caps = ModelCapabilities::for_model(AiModel::StableDiffusionXL);
        assert!(caps.supports_negative_prompt);
        assert!(caps.supports_seed);
        assert_eq!(caps.model, AiModel::StableDiffusionXL);
    }

    #[test]
    fn test_prompt_validation() {
        let caps = ModelCapabilities::for_model(AiModel::StableDiffusion21);

        assert!(caps.validate_prompt("short prompt").is_ok());
        assert!(caps
            .validate_prompt(&"a".repeat(1001))
            .is_err());
    }
}
