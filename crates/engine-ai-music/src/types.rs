// Types for ACE-Step API

use serde::{Deserialize, Serialize};
use std::fmt;

/// Error type for ACE-Step operations
#[derive(Debug)]
pub enum AceStepError {
    /// Network or connection error
    NetworkError(String),
    /// API returned an error
    ApiError(String),
    /// Invalid request parameters
    InvalidRequest(String),
    /// Timeout waiting for generation
    Timeout,
    /// Service not available
    ServiceUnavailable,
}

impl fmt::Display for AceStepError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NetworkError(msg) => write!(f, "Network error: {}", msg),
            Self::ApiError(msg) => write!(f, "API error: {}", msg),
            Self::InvalidRequest(msg) => write!(f, "Invalid request: {}", msg),
            Self::Timeout => write!(f, "Request timed out"),
            Self::ServiceUnavailable => write!(f, "ACE-Step service not available"),
        }
    }
}

impl std::error::Error for AceStepError {}

/// Music generation request parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MusicGenerationRequest {
    /// Text description of the music to generate
    pub prompt: String,

    /// Duration of the music
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<MusicDuration>,

    /// Music style/genre
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<MusicStyle>,

    /// Tempo (BPM)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tempo: Option<u32>,

    /// Whether to include vocals
    #[serde(default)]
    pub instrumental: bool,

    /// Random seed for reproducibility
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<u64>,

    /// Number of inference steps (higher = better quality, slower)
    #[serde(default = "default_steps")]
    pub inference_steps: u32,
}

fn default_steps() -> u32 {
    27 // ACE-Step default
}

impl MusicGenerationRequest {
    /// Create a new request with a text prompt
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            duration: None,
            style: None,
            tempo: None,
            instrumental: false,
            seed: None,
            inference_steps: default_steps(),
        }
    }

    /// Set duration
    pub fn with_duration(mut self, duration: MusicDuration) -> Self {
        self.duration = Some(duration);
        self
    }

    /// Set style/genre
    pub fn with_style(mut self, style: MusicStyle) -> Self {
        self.style = Some(style);
        self
    }

    /// Set tempo (BPM)
    pub fn with_tempo(mut self, tempo: u32) -> Self {
        self.tempo = Some(tempo);
        self
    }

    /// Make it instrumental (no vocals)
    pub fn instrumental(mut self) -> Self {
        self.instrumental = true;
        self
    }

    /// Set random seed
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self
    }

    /// Set inference steps
    pub fn with_steps(mut self, steps: u32) -> Self {
        self.inference_steps = steps;
        self
    }
}

/// Music duration presets
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MusicDuration {
    /// ~15 seconds
    Short,
    /// ~30 seconds
    Medium,
    /// ~60 seconds
    Long,
    /// ~2 minutes
    Extended,
    /// Custom duration in seconds
    Custom(u32),
}

impl MusicDuration {
    /// Get duration in seconds
    pub fn as_secs(&self) -> u32 {
        match self {
            Self::Short => 15,
            Self::Medium => 30,
            Self::Long => 60,
            Self::Extended => 120,
            Self::Custom(secs) => *secs,
        }
    }
}

/// Music style/genre
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MusicStyle {
    Rock,
    Pop,
    Electronic,
    Jazz,
    Classical,
    HipHop,
    Ambient,
    Cinematic,
    Folk,
    Metal,
    Indie,
    /// Custom style description
    Custom(String),
}

impl fmt::Display for MusicStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Rock => write!(f, "rock"),
            Self::Pop => write!(f, "pop"),
            Self::Electronic => write!(f, "electronic"),
            Self::Jazz => write!(f, "jazz"),
            Self::Classical => write!(f, "classical"),
            Self::HipHop => write!(f, "hip-hop"),
            Self::Ambient => write!(f, "ambient"),
            Self::Cinematic => write!(f, "cinematic"),
            Self::Folk => write!(f, "folk"),
            Self::Metal => write!(f, "metal"),
            Self::Indie => write!(f, "indie"),
            Self::Custom(s) => write!(f, "{}", s),
        }
    }
}

/// Result of music generation
#[derive(Debug, Clone)]
pub struct GenerationResult {
    /// Generated audio data (WAV format)
    pub audio_data: Vec<u8>,

    /// Duration of generated audio in seconds
    pub duration_secs: f32,

    /// Sample rate
    pub sample_rate: u32,

    /// Generation metadata
    pub metadata: GenerationMetadata,
}

/// Metadata about the generation process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationMetadata {
    /// Prompt used
    pub prompt: String,

    /// Seed used (if any)
    pub seed: Option<u64>,

    /// Inference steps used
    pub inference_steps: u32,

    /// Generation time in seconds
    pub generation_time_secs: Option<f32>,
}

/// API response from ACE-Step
#[derive(Debug, Deserialize)]
pub(crate) struct ApiResponse {
    pub success: bool,
    pub audio_url: Option<String>,
    pub error: Option<String>,
    pub metadata: Option<GenerationMetadata>,
}

/// Health check response
#[derive(Debug, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub model_loaded: bool,
    pub device: String,
}
