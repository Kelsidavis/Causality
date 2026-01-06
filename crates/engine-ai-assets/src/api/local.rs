// Local Stable Diffusion service client

use super::ApiClient;
use anyhow::{anyhow, Result};
use serde::Serialize;

/// Local Stable Diffusion service client
pub struct LocalClient {
    api_url: String,
    timeout: std::time::Duration,
}

#[derive(Debug, Serialize)]
struct LocalTextureRequest {
    prompt: String,
    negative_prompt: Option<String>,
    width: u32,
    height: u32,
    num_inference_steps: u32,
    guidance_scale: f32,
    seed: Option<u64>,
}

impl LocalClient {
    /// Create a new local client
    pub fn new(api_url: String, timeout_seconds: u64) -> Self {
        Self {
            api_url,
            timeout: std::time::Duration::from_secs(timeout_seconds),
        }
    }

    /// Create a client pointing to localhost
    pub fn localhost(port: u16, timeout_seconds: u64) -> Self {
        Self::new(format!("http://localhost:{}", port), timeout_seconds)
    }

    /// Check if the service is accessible
    async fn check_health(&self) -> Result<()> {
        let client = reqwest::Client::new();
        let health_url = format!("{}/health", self.api_url);

        let response = client
            .get(&health_url)
            .timeout(self.timeout)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(anyhow!(
                "Service health check failed: {}",
                response.status()
            ))
        }
    }
}

#[async_trait::async_trait]
impl ApiClient for LocalClient {
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

        let request = LocalTextureRequest {
            prompt: prompt.to_string(),
            negative_prompt: Some("blurry, low quality, distorted, ugly".to_string()),
            width,
            height,
            num_inference_steps: 50,
            guidance_scale: 7.5,
            seed,
        };

        log::info!("Generating image: {} ({}x{})", prompt, width, height);

        let client = reqwest::Client::new();
        let generate_url = format!("{}/generate-texture", self.api_url);

        let response = client
            .post(&generate_url)
            .timeout(self.timeout)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow!("API error {}: {}", status, text));
        }

        // Parse response to get image name
        let response_json = response.json::<serde_json::Value>().await?;

        if !response_json
            .get("success")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            return Err(anyhow!("Generation failed"));
        }

        if let Some(image_name) = response_json.get("image_name").and_then(|v| v.as_str()) {
            // Fetch the generated image
            let image_url = format!("{}/texture/{}", self.api_url, image_name);
            let image_response = client
                .get(&image_url)
                .timeout(self.timeout)
                .send()
                .await?;

            if !image_response.status().is_success() {
                return Err(anyhow!("Failed to fetch generated image"));
            }

            let image_bytes = image_response.bytes().await?;

            if image_bytes.is_empty() {
                return Err(anyhow!("Empty response from API"));
            }

            log::info!("Successfully generated image ({} bytes)", image_bytes.len());

            Ok(image_bytes.to_vec())
        } else {
            Err(anyhow!("No image_name in response"))
        }
    }

    fn model_name(&self) -> &str {
        "local-stable-diffusion-2-1"
    }

    async fn health_check(&self) -> Result<()> {
        self.check_health().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_localhost_client_creation() {
        let client = LocalClient::localhost(8000, 300);
        assert_eq!(client.api_url, "http://localhost:8000");
        assert_eq!(client.model_name(), "local-stable-diffusion-2-1");
    }

    #[test]
    fn test_custom_url_client_creation() {
        let client = LocalClient::new("http://192.168.1.100:8000".to_string(), 300);
        assert_eq!(client.api_url, "http://192.168.1.100:8000");
    }
}
