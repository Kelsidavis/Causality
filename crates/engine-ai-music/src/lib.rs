// AI Music Generation - ACE-Step Integration
//
// This crate provides integration with ACE-Step, an AI music generation model.
// It communicates with an ACE-Step API service (running separately) via HTTP.
//
// Setup: Run ACE-Step server with `acestep --port 7865`
// Then use this client to generate music on demand.


pub mod client;
pub mod types;

pub use client::AceStepClient;
pub use types::*;

/// Configuration for ACE-Step service
#[derive(Debug, Clone)]
pub struct AceStepConfig {
    /// Base URL of the ACE-Step API service
    pub base_url: String,
    /// Default timeout for requests (in seconds)
    pub timeout_secs: u64,
}

impl Default for AceStepConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:7865".to_string(),
            timeout_secs: 300, // 5 minutes for music generation
        }
    }
}

impl AceStepConfig {
    /// Create configuration with custom URL
    pub fn with_url(url: impl Into<String>) -> Self {
        Self {
            base_url: url.into(),
            ..Default::default()
        }
    }

    /// Set timeout in seconds
    pub fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.timeout_secs = timeout_secs;
        self
    }
}

/// Convenience re-exports
pub mod prelude {
    pub use super::{
        AceStepClient, AceStepConfig, MusicGenerationRequest, MusicStyle, MusicDuration,
        GenerationResult, AceStepError,
    };
}
