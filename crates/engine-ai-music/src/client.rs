// ACE-Step API Client

use super::{
    AceStepConfig, AceStepError, GenerationMetadata, GenerationResult, HealthResponse,
    MusicGenerationRequest,
};
use reqwest::Client;
use std::path::Path;
use std::time::Duration;

/// Client for ACE-Step music generation API
pub struct AceStepClient {
    config: AceStepConfig,
    client: Client,
}

impl AceStepClient {
    /// Create a new client with default configuration
    pub fn new() -> Self {
        Self::with_config(AceStepConfig::default())
    }

    /// Create a client with custom configuration
    pub fn with_config(config: AceStepConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .expect("Failed to create HTTP client");

        Self { config, client }
    }

    /// Check if the ACE-Step service is available
    pub async fn health_check(&self) -> Result<HealthResponse, AceStepError> {
        let url = format!("{}/health", self.config.base_url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| AceStepError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(AceStepError::ServiceUnavailable);
        }

        response
            .json::<HealthResponse>()
            .await
            .map_err(|e| AceStepError::ApiError(e.to_string()))
    }

    /// Generate music from a text prompt
    pub async fn generate_music(
        &self,
        request: MusicGenerationRequest,
    ) -> Result<GenerationResult, AceStepError> {
        log::info!("Generating music: '{}'", request.prompt);

        let url = format!("{}/generate", self.config.base_url);

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    AceStepError::Timeout
                } else {
                    AceStepError::NetworkError(e.to_string())
                }
            })?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AceStepError::ApiError(error_text));
        }

        let api_response: super::types::ApiResponse = response
            .json()
            .await
            .map_err(|e| AceStepError::ApiError(e.to_string()))?;

        if !api_response.success {
            let error_msg = api_response
                .error
                .unwrap_or_else(|| "Unknown error".to_string());
            return Err(AceStepError::ApiError(error_msg));
        }

        // Download the generated audio
        let audio_url = api_response
            .audio_url
            .ok_or_else(|| AceStepError::ApiError("No audio URL in response".to_string()))?;

        let audio_data = self
            .client
            .get(&audio_url)
            .send()
            .await
            .map_err(|e| AceStepError::NetworkError(e.to_string()))?
            .bytes()
            .await
            .map_err(|e| AceStepError::NetworkError(e.to_string()))?
            .to_vec();

        log::info!(
            "Music generated successfully ({} bytes)",
            audio_data.len()
        );

        Ok(GenerationResult {
            audio_data,
            duration_secs: request
                .duration
                .map(|d| d.as_secs() as f32)
                .unwrap_or(30.0),
            sample_rate: 44100, // ACE-Step default
            metadata: api_response.metadata.unwrap_or(GenerationMetadata {
                prompt: request.prompt.clone(),
                seed: request.seed,
                inference_steps: request.inference_steps,
                generation_time_secs: None,
            }),
        })
    }

    /// Generate music and save to file
    pub async fn generate_and_save(
        &self,
        request: MusicGenerationRequest,
        output_path: impl AsRef<Path>,
    ) -> Result<(), AceStepError> {
        let result = self.generate_music(request).await?;

        std::fs::write(output_path.as_ref(), &result.audio_data)
            .map_err(|e| AceStepError::ApiError(format!("Failed to save audio: {}", e)))?;

        log::info!("Saved generated music to {:?}", output_path.as_ref());

        Ok(())
    }

    /// Generate a variation of existing audio
    pub async fn generate_variation(
        &self,
        original_audio: &[u8],
        variation_strength: f32, // 0.0 to 1.0
    ) -> Result<GenerationResult, AceStepError> {
        let url = format!("{}/variation", self.config.base_url);

        let form = reqwest::multipart::Form::new()
            .part(
                "audio",
                reqwest::multipart::Part::bytes(original_audio.to_vec())
                    .file_name("audio.wav")
                    .mime_str("audio/wav")
                    .map_err(|e| AceStepError::InvalidRequest(e.to_string()))?,
            )
            .text("strength", variation_strength.to_string());

        let response = self
            .client
            .post(&url)
            .multipart(form)
            .send()
            .await
            .map_err(|e| AceStepError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(AceStepError::ApiError(
                response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".to_string()),
            ));
        }

        let audio_data = response
            .bytes()
            .await
            .map_err(|e| AceStepError::NetworkError(e.to_string()))?
            .to_vec();

        Ok(GenerationResult {
            audio_data,
            duration_secs: 30.0,
            sample_rate: 44100,
            metadata: GenerationMetadata {
                prompt: format!("Variation (strength: {})", variation_strength),
                seed: None,
                inference_steps: 27,
                generation_time_secs: None,
            },
        })
    }

    /// Extend existing audio (add music before or after)
    pub async fn extend_audio(
        &self,
        original_audio: &[u8],
        extend_before: bool,
        duration_secs: u32,
    ) -> Result<GenerationResult, AceStepError> {
        let url = format!("{}/extend", self.config.base_url);

        let form = reqwest::multipart::Form::new()
            .part(
                "audio",
                reqwest::multipart::Part::bytes(original_audio.to_vec())
                    .file_name("audio.wav")
                    .mime_str("audio/wav")
                    .map_err(|e| AceStepError::InvalidRequest(e.to_string()))?,
            )
            .text("direction", if extend_before { "before" } else { "after" })
            .text("duration", duration_secs.to_string());

        let response = self
            .client
            .post(&url)
            .multipart(form)
            .send()
            .await
            .map_err(|e| AceStepError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(AceStepError::ApiError(
                response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".to_string()),
            ));
        }

        let audio_data = response
            .bytes()
            .await
            .map_err(|e| AceStepError::NetworkError(e.to_string()))?
            .to_vec();

        Ok(GenerationResult {
            audio_data,
            duration_secs: duration_secs as f32,
            sample_rate: 44100,
            metadata: GenerationMetadata {
                prompt: format!(
                    "Extended {} by {}s",
                    if extend_before { "before" } else { "after" },
                    duration_secs
                ),
                seed: None,
                inference_steps: 27,
                generation_time_secs: None,
            },
        })
    }
}

impl Default for AceStepClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Only run when ACE-Step service is running
    async fn test_health_check() {
        let client = AceStepClient::new();
        let health = client.health_check().await;
        assert!(health.is_ok());
    }

    #[tokio::test]
    #[ignore] // Only run when ACE-Step service is running
    async fn test_generate_music() {
        let client = AceStepClient::new();

        let request = MusicGenerationRequest::new("Epic cinematic battle music")
            .with_duration(super::super::MusicDuration::Short)
            .with_style(super::super::MusicStyle::Cinematic)
            .instrumental();

        let result = client.generate_music(request).await;
        assert!(result.is_ok());

        let audio = result.unwrap();
        assert!(!audio.audio_data.is_empty());
    }
}
