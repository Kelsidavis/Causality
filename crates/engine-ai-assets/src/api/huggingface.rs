// Hugging Face Inference API client for Stable Diffusion

use super::ApiClient;
use anyhow::{anyhow, Result};
use serde::Serialize;
use std::time::Duration;

const HF_API_URL: &str = "https://api-inference.huggingface.co/models";

/// Hugging Face API client
pub struct HuggingFaceClient {
    api_key: String,
    model: String,
    client: reqwest::Client,
    timeout: Duration,
}

#[derive(Debug, Serialize)]
struct HFRequest {
    inputs: String,
    parameters: HFParameters,
}

#[derive(Debug, Serialize)]
struct HFParameters {
    negative_prompt: Option<String>,
    height: u32,
    width: u32,
    num_inference_steps: Option<u32>,
    guidance_scale: Option<f32>,
    seed: Option<i64>,
}

impl HuggingFaceClient {
    /// Create a new Hugging Face client
    pub fn new(api_key: String, model: String, timeout_seconds: u64) -> Self {
        Self {
            api_key,
            model,
            client: reqwest::Client::new(),
            timeout: Duration::from_secs(timeout_seconds),
        }
    }

    /// Get the full API endpoint URL
    fn endpoint(&self) -> String {
        format!("{}/{}", HF_API_URL, self.model)
    }
}

#[async_trait::async_trait]
impl ApiClient for HuggingFaceClient {
    async fn generate_image(
        &self,
        prompt: &str,
        width: u32,
        height: u32,
        seed: Option<u64>,
    ) -> Result<Vec<u8>> {
        // Validate dimensions
        if width < 256 || width > 2048 || height < 256 || height > 2048 {
            return Err(anyhow!("Image dimensions must be between 256 and 2048"));
        }

        // Round to nearest multiple of 64 (Stable Diffusion requirement)
        let width = ((width + 31) / 64) * 64;
        let height = ((height + 31) / 64) * 64;

        let request = HFRequest {
            inputs: prompt.to_string(),
            parameters: HFParameters {
                negative_prompt: Some("blurry, low quality, distorted, ugly".to_string()),
                height,
                width,
                num_inference_steps: Some(50),
                guidance_scale: Some(7.5),
                seed: seed.map(|s| s as i64),
            },
        };

        log::info!("Generating image: {} ({}x{})", prompt, width, height);

        let response = self
            .client
            .post(self.endpoint())
            .header("Authorization", format!("Bearer {}", self.api_key))
            .timeout(self.timeout)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow!("API error {}: {}", status, text));
        }

        let image_bytes = response.bytes().await?;

        if image_bytes.is_empty() {
            return Err(anyhow!("Empty response from API"));
        }

        log::info!("Successfully generated image ({} bytes)", image_bytes.len());

        Ok(image_bytes.to_vec())
    }

    fn model_name(&self) -> &str {
        &self.model
    }

    async fn health_check(&self) -> Result<()> {
        let response = self
            .client
            .get(self.endpoint())
            .header("Authorization", format!("Bearer {}", self.api_key))
            .timeout(Duration::from_secs(10))
            .send()
            .await?;

        if response.status().is_success() || response.status() == 429 {
            // 429 means the model is busy, which is fine - it's available
            Ok(())
        } else {
            Err(anyhow!("API health check failed: {}", response.status()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = HuggingFaceClient::new(
            "test_key".to_string(),
            "stabilityai/stable-diffusion-2-1".to_string(),
            300,
        );
        assert_eq!(client.model_name(), "stabilityai/stable-diffusion-2-1");
    }

    #[test]
    fn test_endpoint_url() {
        let client = HuggingFaceClient::new(
            "test_key".to_string(),
            "stabilityai/stable-diffusion-2-1".to_string(),
            300,
        );
        assert!(client.endpoint().contains("api-inference.huggingface.co"));
    }
}
