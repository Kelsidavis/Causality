// Audio API for scripts

use rhai::Engine;
use std::sync::{Arc, Mutex};

/// Audio command that scripts can issue
#[derive(Debug, Clone)]
pub enum AudioCommand {
    PlaySound { path: String, volume: f32 },
    PlayMusic { path: String, volume: f32, looping: bool },
    StopMusic,
}

/// Thread-safe audio command queue
pub type AudioCommandQueue = Arc<Mutex<Vec<AudioCommand>>>;

/// Register audio functions with Rhai engine
pub fn register_audio_api(engine: &mut Engine, command_queue: AudioCommandQueue) {
    // Clone for each closure
    let queue_clone1 = command_queue.clone();
    let queue_clone2 = command_queue.clone();
    let queue_clone3 = command_queue.clone();

    // Play sound (2D, no position)
    engine.register_fn("play_sound", move |path: &str, volume: f64| {
        let mut queue = queue_clone1.lock().unwrap();
        queue.push(AudioCommand::PlaySound {
            path: path.to_string(),
            volume: volume as f32,
        });
        true
    });

    // Play music (looping background music)
    engine.register_fn("play_music", move |path: &str, volume: f64, looping: bool| {
        let mut queue = queue_clone2.lock().unwrap();
        queue.push(AudioCommand::PlayMusic {
            path: path.to_string(),
            volume: volume as f32,
            looping,
        });
        true
    });

    // Stop music
    engine.register_fn("stop_music", move || {
        let mut queue = queue_clone3.lock().unwrap();
        queue.push(AudioCommand::StopMusic);
    });
}
