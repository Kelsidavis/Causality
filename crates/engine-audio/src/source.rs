// Audio Source - component for spatial audio attached to entities

use glam::Vec3;

/// Type of sound
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SoundType {
    /// Play once
    OneShot,
    /// Loop continuously
    Looping,
}

/// Audio source component - plays 3D spatial audio
#[derive(Debug, Clone)]
pub struct AudioSource {
    /// Path to audio file
    pub audio_path: String,
    /// Sound type (one-shot or looping)
    pub sound_type: SoundType,
    /// Volume (0.0 to 1.0)
    pub volume: f32,
    /// Maximum hearing distance
    pub max_distance: f32,
    /// Whether the source is currently playing
    pub playing: bool,
    /// Whether to play on start
    pub play_on_start: bool,
}

impl AudioSource {
    /// Create a new audio source
    pub fn new(audio_path: String) -> Self {
        Self {
            audio_path,
            sound_type: SoundType::OneShot,
            volume: 1.0,
            max_distance: 50.0,
            playing: false,
            play_on_start: false,
        }
    }

    /// Set sound type
    pub fn with_sound_type(mut self, sound_type: SoundType) -> Self {
        self.sound_type = sound_type;
        self
    }

    /// Set volume
    pub fn with_volume(mut self, volume: f32) -> Self {
        self.volume = volume.clamp(0.0, 1.0);
        self
    }

    /// Set max distance
    pub fn with_max_distance(mut self, max_distance: f32) -> Self {
        self.max_distance = max_distance.max(0.1);
        self
    }

    /// Set play on start
    pub fn with_play_on_start(mut self, play_on_start: bool) -> Self {
        self.play_on_start = play_on_start;
        self
    }

    /// Start playing
    pub fn play(&mut self) {
        self.playing = true;
    }

    /// Stop playing
    pub fn stop(&mut self) {
        self.playing = false;
    }
}

impl Default for AudioSource {
    fn default() -> Self {
        Self::new(String::new())
    }
}
