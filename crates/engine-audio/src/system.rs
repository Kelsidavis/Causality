// Audio System - manages audio output and playback

use anyhow::{Context, Result};
use glam::Vec3;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::listener::AudioListener;
use crate::source::{AudioSource, SoundType};

/// Cached audio data
pub struct AudioData {
    pub data: Arc<Vec<u8>>,
}

/// Audio system - manages audio playback and 3D spatial audio
pub struct AudioSystem {
    /// Audio output stream
    _stream: OutputStream,
    /// Audio output stream handle
    stream_handle: OutputStreamHandle,
    /// Asset root directory
    asset_root: PathBuf,
    /// Cached audio files
    cache: HashMap<PathBuf, AudioData>,
    /// Active music sink
    music_sink: Option<Sink>,
    /// Active sound effects
    active_sounds: Vec<Sink>,
    /// Global volume (0.0 to 1.0)
    master_volume: f32,
}

impl AudioSystem {
    /// Create a new audio system
    pub fn new<P: AsRef<Path>>(asset_root: P) -> Result<Self> {
        let (_stream, stream_handle) = OutputStream::try_default()
            .context("Failed to create audio output stream")?;

        Ok(Self {
            _stream,
            stream_handle,
            asset_root: asset_root.as_ref().to_path_buf(),
            cache: HashMap::new(),
            music_sink: None,
            active_sounds: Vec::new(),
            master_volume: 1.0,
        })
    }

    /// Get the full path for an audio asset
    fn full_path(&self, relative_path: &str) -> PathBuf {
        self.asset_root.join(relative_path)
    }

    /// Load audio data from file (with caching)
    fn load_audio_data(&mut self, path: &str) -> Result<AudioData> {
        let full_path = self.full_path(path);

        // Check cache
        if let Some(data) = self.cache.get(&full_path) {
            return Ok(AudioData {
                data: Arc::clone(&data.data),
            });
        }

        // Load from disk
        log::info!("Loading audio: {:?}", full_path);
        let file = File::open(&full_path)
            .with_context(|| format!("Failed to open audio file: {}", path))?;

        let mut reader = BufReader::new(file);
        let mut data = Vec::new();
        std::io::copy(&mut reader, &mut data)?;

        let audio_data = AudioData {
            data: Arc::new(data),
        };

        self.cache.insert(full_path, AudioData {
            data: Arc::clone(&audio_data.data),
        });

        Ok(audio_data)
    }

    /// Play a sound effect (non-looping)
    pub fn play_sound(&mut self, path: &str, volume: f32) -> Result<()> {
        let audio_data = self.load_audio_data(path)?;
        let cursor = std::io::Cursor::new((*audio_data.data).clone());

        let source = Decoder::new(cursor)
            .with_context(|| format!("Failed to decode audio: {}", path))?;

        let sink = Sink::try_new(&self.stream_handle)?;
        sink.set_volume(volume * self.master_volume);
        sink.append(source);
        sink.detach();

        Ok(())
    }

    /// Play background music (looping)
    pub fn play_music(&mut self, path: &str, volume: f32, looping: bool) -> Result<()> {
        // Stop current music if playing
        if let Some(sink) = self.music_sink.take() {
            sink.stop();
        }

        let audio_data = self.load_audio_data(path)?;
        let cursor = std::io::Cursor::new((*audio_data.data).clone());

        let source = Decoder::new(cursor)
            .with_context(|| format!("Failed to decode audio: {}", path))?;

        let sink = Sink::try_new(&self.stream_handle)?;
        sink.set_volume(volume * self.master_volume);

        if looping {
            sink.append(source.repeat_infinite());
        } else {
            sink.append(source);
        }

        self.music_sink = Some(sink);

        Ok(())
    }

    /// Stop background music
    pub fn stop_music(&mut self) {
        if let Some(sink) = self.music_sink.take() {
            sink.stop();
        }
    }

    /// Pause background music
    pub fn pause_music(&mut self) {
        if let Some(sink) = &self.music_sink {
            sink.pause();
        }
    }

    /// Resume background music
    pub fn resume_music(&mut self) {
        if let Some(sink) = &self.music_sink {
            sink.play();
        }
    }

    /// Set master volume (0.0 to 1.0)
    pub fn set_master_volume(&mut self, volume: f32) {
        self.master_volume = volume.clamp(0.0, 1.0);

        // Update music volume
        if let Some(sink) = &self.music_sink {
            sink.set_volume(self.master_volume);
        }
    }

    /// Get master volume
    pub fn master_volume(&self) -> f32 {
        self.master_volume
    }

    /// Play a 3D spatial sound
    pub fn play_3d_sound(
        &mut self,
        path: &str,
        position: Vec3,
        listener: &AudioListener,
        volume: f32,
        max_distance: f32,
    ) -> Result<()> {
        let audio_data = self.load_audio_data(path)?;
        let cursor = std::io::Cursor::new((*audio_data.data).clone());

        let source = Decoder::new(cursor)
            .with_context(|| format!("Failed to decode audio: {}", path))?;

        // Calculate distance attenuation
        let distance = (position - listener.position).length();
        let attenuation = if distance < max_distance {
            1.0 - (distance / max_distance).powi(2)
        } else {
            0.0
        };

        let final_volume = (volume * attenuation * self.master_volume).clamp(0.0, 1.0);

        if final_volume > 0.01 {
            let sink = Sink::try_new(&self.stream_handle)?;
            sink.set_volume(final_volume);
            sink.append(source);
            sink.detach();
        }

        Ok(())
    }

    /// Update audio system (call each frame to clean up finished sounds)
    pub fn update(&mut self) {
        // Remove finished sound effects
        self.active_sounds.retain(|sink| !sink.empty());
    }

    /// Get cache size
    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }

    /// Clear cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
        log::info!("Audio cache cleared");
    }
}

impl Default for AudioSystem {
    fn default() -> Self {
        Self::new("assets").expect("Failed to create default audio system")
    }
}
