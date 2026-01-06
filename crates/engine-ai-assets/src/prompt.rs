// Prompt optimization for better AI-generated results

/// Prompt optimizer for enhancing image generation quality
pub struct PromptOptimizer {
    quality_level: QualityLevel,
}

/// Quality level for image generation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QualityLevel {
    Fast,
    Standard,
    High,
    Best,
}

impl QualityLevel {
    fn inference_steps(&self) -> u32 {
        match self {
            Self::Fast => 20,
            Self::Standard => 35,
            Self::High => 50,
            Self::Best => 75,
        }
    }

    fn guidance_scale(&self) -> f32 {
        match self {
            Self::Fast => 5.0,
            Self::Standard => 7.0,
            Self::High => 7.5,
            Self::Best => 8.5,
        }
    }
}

impl PromptOptimizer {
    /// Create a new prompt optimizer
    pub fn new(quality_level: QualityLevel) -> Self {
        Self { quality_level }
    }

    /// Optimize a prompt for better results
    pub fn optimize(&self, prompt: &str) -> OptimizedPrompt {
        let base_prompt = self.enhance_prompt(prompt);
        let negative_prompt = self.default_negative_prompt();

        OptimizedPrompt {
            prompt: base_prompt,
            negative_prompt,
            steps: self.quality_level.inference_steps(),
            guidance_scale: self.quality_level.guidance_scale(),
        }
    }

    /// Enhance the prompt with quality keywords
    fn enhance_prompt(&self, prompt: &str) -> String {
        let mut enhanced = prompt.to_string();

        // Add quality modifiers based on level
        match self.quality_level {
            QualityLevel::Fast => {
                if !enhanced.contains("fast") {
                    enhanced.push_str(", fast render");
                }
            }
            QualityLevel::Standard => {
                if !enhanced.contains("high quality") && !enhanced.contains("detailed") {
                    enhanced.push_str(", high quality, detailed");
                }
            }
            QualityLevel::High => {
                if !enhanced.contains("high quality") {
                    enhanced.push_str(", high quality");
                }
                if !enhanced.contains("detailed") {
                    enhanced.push_str(", highly detailed");
                }
                if !enhanced.contains("professional") {
                    enhanced.push_str(", professional");
                }
            }
            QualityLevel::Best => {
                if !enhanced.contains("masterpiece") {
                    enhanced.push_str(", masterpiece");
                }
                if !enhanced.contains("best quality") {
                    enhanced.push_str(", best quality");
                }
                if !enhanced.contains("ultra detailed") {
                    enhanced.push_str(", ultra detailed");
                }
                if !enhanced.contains("professional") {
                    enhanced.push_str(", professional");
                }
            }
        }

        enhanced
    }

    /// Get default negative prompt
    fn default_negative_prompt(&self) -> String {
        match self.quality_level {
            QualityLevel::Fast => {
                "blurry, low quality".to_string()
            }
            QualityLevel::Standard => {
                "blurry, low quality, distorted, ugly, watermark".to_string()
            }
            QualityLevel::High => {
                "blurry, low quality, distorted, ugly, watermark, artifacts, amateur".to_string()
            }
            QualityLevel::Best => {
                "blurry, low quality, distorted, ugly, watermark, artifacts, amateur, bad anatomy, bad hands".to_string()
            }
        }
    }
}

/// Result of prompt optimization
#[derive(Debug, Clone)]
pub struct OptimizedPrompt {
    pub prompt: String,
    pub negative_prompt: String,
    pub steps: u32,
    pub guidance_scale: f32,
}

/// Style modifiers for specialized asset types
pub mod styles {
    /// Photorealistic texture styles
    pub fn photorealistic() -> &'static str {
        "photorealistic, physically based, high fidelity"
    }

    /// Game art texture styles
    pub fn game_art() -> &'static str {
        "game asset, stylized, clean, crisp edges"
    }

    /// PBR (Physically Based Rendering) texture styles
    pub fn pbr_material() -> &'static str {
        "pbr texture, 4k, seamless, tileable, no seams"
    }

    /// Normal map styles
    pub fn normal_map() -> &'static str {
        "normal map, smooth gradients, blue-ish tones, no artifacts"
    }

    /// Roughness map styles
    pub fn roughness_map() -> &'static str {
        "roughness map, grayscale, varied surface detail"
    }

    /// Skybox/environment styles
    pub fn skybox() -> &'static str {
        "360 degree panorama, environment map, seamless wrap"
    }

    /// Character texture styles
    pub fn character_skin() -> &'static str {
        "character skin texture, realistic, subtle detail, clean"
    }

    /// Wood texture styles
    pub fn wood() -> &'static str {
        "wood texture, grain pattern, realistic aging"
    }

    /// Metal texture styles
    pub fn metal() -> &'static str {
        "metal texture, brushed finish, realistic wear"
    }

    /// Stone texture styles
    pub fn stone() -> &'static str {
        "stone texture, natural surface variation, worn edges"
    }
}

/// Prompt templates for common asset types
pub mod templates {
    /// Template for PBR texture generation
    pub fn pbr_texture(material_type: &str, quality: &str) -> String {
        format!(
            "{} material texture, {}, seamless, 4k, pbr, physically based rendering, detailed surface",
            material_type, quality
        )
    }

    /// Template for skybox generation
    pub fn skybox(environment: &str, time_of_day: &str) -> String {
        format!(
            "beautiful skybox, {}, {}, 360 panorama, high quality lighting, photorealistic",
            environment, time_of_day
        )
    }

    /// Template for game-ready texture
    pub fn game_texture(object_type: &str, style: &str) -> String {
        format!(
            "game-ready {} texture, {}, clean, tileable, optimized for games, high quality",
            object_type, style
        )
    }

    /// Template for normal map
    pub fn normal_map(surface_type: &str) -> String {
        format!(
            "{} normal map, detailed relief, tangent space normal, no seams",
            surface_type
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quality_levels_inference_steps() {
        assert_eq!(QualityLevel::Fast.inference_steps(), 20);
        assert_eq!(QualityLevel::Standard.inference_steps(), 35);
        assert_eq!(QualityLevel::High.inference_steps(), 50);
        assert_eq!(QualityLevel::Best.inference_steps(), 75);
    }

    #[test]
    fn test_quality_levels_guidance() {
        assert_eq!(QualityLevel::Fast.guidance_scale(), 5.0);
        assert_eq!(QualityLevel::Standard.guidance_scale(), 7.0);
        assert_eq!(QualityLevel::High.guidance_scale(), 7.5);
        assert_eq!(QualityLevel::Best.guidance_scale(), 8.5);
    }

    #[test]
    fn test_optimizer_enhances_prompt() {
        let optimizer = PromptOptimizer::new(QualityLevel::High);
        let optimized = optimizer.optimize("stone wall");

        assert!(optimized.prompt.contains("stone wall"));
        assert!(optimized.prompt.contains("high quality"));
        assert_eq!(optimized.steps, 50);
    }

    #[test]
    fn test_negative_prompt_empty_for_fast() {
        let optimizer = PromptOptimizer::new(QualityLevel::Fast);
        let optimized = optimizer.optimize("test");

        assert!(!optimized.negative_prompt.is_empty());
        assert!(optimized.negative_prompt.contains("blurry"));
    }

    #[test]
    fn test_pbr_template() {
        let template = templates::pbr_texture("wood", "weathered");
        assert!(template.contains("wood"));
        assert!(template.contains("weathered"));
        assert!(template.contains("pbr"));
    }
}
