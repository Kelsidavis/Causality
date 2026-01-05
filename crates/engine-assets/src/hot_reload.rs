// Hot reload system - monitors files for changes

use anyhow::Result;
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::{Duration, Instant};

pub struct HotReloadWatcher {
    _watcher: RecommendedWatcher,
    event_receiver: Receiver<notify::Result<Event>>,
    debounce_map: std::collections::HashMap<PathBuf, Instant>,
    debounce_duration: Duration,
}

#[derive(Debug, Clone)]
pub enum ReloadEvent {
    ScriptChanged(PathBuf),
    AssetChanged(PathBuf),
    TextureChanged(PathBuf),
    ModelChanged(PathBuf),
}

impl HotReloadWatcher {
    pub fn new() -> Result<Self> {
        let (tx, rx) = channel();
        let watcher = notify::recommended_watcher(tx)?;

        Ok(Self {
            _watcher: watcher,
            event_receiver: rx,
            debounce_map: std::collections::HashMap::new(),
            debounce_duration: Duration::from_millis(100),
        })
    }

    pub fn watch_directory<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        self._watcher
            .watch(path.as_ref(), RecursiveMode::Recursive)?;
        log::info!("Watching directory: {:?}", path.as_ref());
        Ok(())
    }

    pub fn watch_file<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        self._watcher
            .watch(path.as_ref(), RecursiveMode::NonRecursive)?;
        log::info!("Watching file: {:?}", path.as_ref());
        Ok(())
    }

    pub fn poll_events(&mut self) -> Vec<ReloadEvent> {
        let mut events = Vec::new();
        let now = Instant::now();

        // Process all available events
        while let Ok(result) = self.event_receiver.try_recv() {
            match result {
                Ok(event) => {
                    if let Some(reload_event) = self.process_event(event, now) {
                        events.push(reload_event);
                    }
                }
                Err(e) => {
                    log::error!("File watcher error: {}", e);
                }
            }
        }

        events
    }

    fn process_event(&mut self, event: Event, now: Instant) -> Option<ReloadEvent> {
        match event.kind {
            EventKind::Modify(_) | EventKind::Create(_) => {
                for path in event.paths {
                    // Check debounce
                    if let Some(last_event) = self.debounce_map.get(&path) {
                        if now.duration_since(*last_event) < self.debounce_duration {
                            continue; // Skip this event, too soon
                        }
                    }

                    // Update debounce time
                    self.debounce_map.insert(path.clone(), now);

                    // Classify the file type and create appropriate reload event
                    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                        return match ext {
                            "rhai" => {
                                log::info!("Script changed: {:?}", path);
                                Some(ReloadEvent::ScriptChanged(path))
                            }
                            "gltf" | "glb" => {
                                log::info!("Model changed: {:?}", path);
                                Some(ReloadEvent::ModelChanged(path))
                            }
                            "png" | "jpg" | "jpeg" | "bmp" | "tga" => {
                                log::info!("Texture changed: {:?}", path);
                                Some(ReloadEvent::TextureChanged(path))
                            }
                            _ => {
                                log::debug!("Asset changed: {:?}", path);
                                Some(ReloadEvent::AssetChanged(path))
                            }
                        };
                    }
                }
            }
            _ => {}
        }

        None
    }

    pub fn cleanup_old_debounce_entries(&mut self) {
        let now = Instant::now();
        let timeout = Duration::from_secs(10);

        self.debounce_map
            .retain(|_, last_time| now.duration_since(*last_time) < timeout);
    }
}

impl Default for HotReloadWatcher {
    fn default() -> Self {
        Self::new().expect("Failed to create hot reload watcher")
    }
}
