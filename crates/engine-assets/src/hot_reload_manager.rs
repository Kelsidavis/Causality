// Hot reload manager - coordinates asset reloading

use crate::hot_reload::{HotReloadWatcher, ReloadEvent};
use crate::manager::AssetManager;
use anyhow::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Tracks which assets are used by the application
#[derive(Debug, Clone)]
pub struct AssetRegistry {
    /// Map from full path to relative path
    textures: HashMap<PathBuf, String>,
    models: HashMap<PathBuf, String>,
    scripts: HashMap<PathBuf, String>,
}

impl AssetRegistry {
    pub fn new() -> Self {
        Self {
            textures: HashMap::new(),
            models: HashMap::new(),
            scripts: HashMap::new(),
        }
    }

    /// Register a texture for hot-reload tracking
    pub fn register_texture(&mut self, full_path: PathBuf, relative_path: String) {
        self.textures.insert(full_path, relative_path);
    }

    /// Register a model for hot-reload tracking
    pub fn register_model(&mut self, full_path: PathBuf, relative_path: String) {
        self.models.insert(full_path, relative_path);
    }

    /// Register a script for hot-reload tracking
    pub fn register_script(&mut self, full_path: PathBuf, relative_path: String) {
        self.scripts.insert(full_path, relative_path);
    }

    /// Get relative path for a texture
    pub fn get_texture_path(&self, full_path: &Path) -> Option<&str> {
        self.textures.get(full_path).map(|s| s.as_str())
    }

    /// Get relative path for a model
    pub fn get_model_path(&self, full_path: &Path) -> Option<&str> {
        self.models.get(full_path).map(|s| s.as_str())
    }

    /// Get relative path for a script
    pub fn get_script_path(&self, full_path: &Path) -> Option<&str> {
        self.scripts.get(full_path).map(|s| s.as_str())
    }

    /// Clear all registrations
    pub fn clear(&mut self) {
        self.textures.clear();
        self.models.clear();
        self.scripts.clear();
    }

    /// Get statistics
    pub fn stats(&self) -> AssetRegistryStats {
        AssetRegistryStats {
            texture_count: self.textures.len(),
            model_count: self.models.len(),
            script_count: self.scripts.len(),
        }
    }
}

impl Default for AssetRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about registered assets
#[derive(Debug, Clone, Copy)]
pub struct AssetRegistryStats {
    pub texture_count: usize,
    pub model_count: usize,
    pub script_count: usize,
}

/// Hot reload manager - coordinates file watching and asset reloading
pub struct HotReloadManager {
    watcher: HotReloadWatcher,
    registry: AssetRegistry,
    enabled: bool,
}

impl HotReloadManager {
    /// Create a new hot reload manager
    pub fn new() -> Result<Self> {
        Ok(Self {
            watcher: HotReloadWatcher::new()?,
            registry: AssetRegistry::new(),
            enabled: true,
        })
    }

    /// Start watching a directory for changes
    pub fn watch_directory<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        self.watcher.watch_directory(path)
    }

    /// Register a texture for tracking
    pub fn register_texture(&mut self, full_path: PathBuf, relative_path: String) {
        self.registry.register_texture(full_path, relative_path);
    }

    /// Register a model for tracking
    pub fn register_model(&mut self, full_path: PathBuf, relative_path: String) {
        self.registry.register_model(full_path, relative_path);
    }

    /// Register a script for tracking
    pub fn register_script(&mut self, full_path: PathBuf, relative_path: String) {
        self.registry.register_script(full_path, relative_path);
    }

    /// Enable or disable hot reloading
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        log::info!("Hot reload {}", if enabled { "enabled" } else { "disabled" });
    }

    /// Check if hot reload is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Poll for file changes and reload assets
    pub fn update(&mut self, asset_manager: &mut AssetManager) -> HotReloadResult {
        if !self.enabled {
            return HotReloadResult::default();
        }

        let events = self.watcher.poll_events();
        let mut result = HotReloadResult::default();

        for event in events {
            match event {
                ReloadEvent::TextureChanged(path) => {
                    if let Some(relative_path) = self.registry.get_texture_path(&path) {
                        match asset_manager.reload_texture(relative_path) {
                            Ok(_) => {
                                log::info!("Successfully reloaded texture: {}", relative_path);
                                result.textures_reloaded += 1;
                            }
                            Err(e) => {
                                log::error!("Failed to reload texture {}: {}", relative_path, e);
                                result.errors.push(format!("Texture reload failed: {}", e));
                            }
                        }
                    }
                }
                ReloadEvent::ModelChanged(path) => {
                    if let Some(relative_path) = self.registry.get_model_path(&path) {
                        match asset_manager.reload_gltf(relative_path) {
                            Ok(_) => {
                                log::info!("Successfully reloaded model: {}", relative_path);
                                result.models_reloaded += 1;
                            }
                            Err(e) => {
                                log::error!("Failed to reload model {}: {}", relative_path, e);
                                result.errors.push(format!("Model reload failed: {}", e));
                            }
                        }
                    }
                }
                ReloadEvent::ScriptChanged(path) => {
                    if let Some(relative_path) = self.registry.get_script_path(&path) {
                        log::info!("Script changed: {}", relative_path);
                        result.scripts_changed.push(relative_path.to_string());
                    }
                }
                ReloadEvent::AssetChanged(path) => {
                    log::debug!("Generic asset changed: {:?}", path);
                }
            }
        }

        // Periodic cleanup of debounce entries
        self.watcher.cleanup_old_debounce_entries();

        result
    }

    /// Get registry statistics
    pub fn stats(&self) -> AssetRegistryStats {
        self.registry.stats()
    }

    /// Clear all registered assets
    pub fn clear_registry(&mut self) {
        self.registry.clear();
    }
}

impl Default for HotReloadManager {
    fn default() -> Self {
        Self::new().expect("Failed to create hot reload manager")
    }
}

/// Result of a hot reload update
#[derive(Debug, Default)]
pub struct HotReloadResult {
    /// Number of textures reloaded
    pub textures_reloaded: usize,
    /// Number of models reloaded
    pub models_reloaded: usize,
    /// Scripts that changed (need manual handling)
    pub scripts_changed: Vec<String>,
    /// Errors that occurred during reload
    pub errors: Vec<String>,
}

impl HotReloadResult {
    /// Check if any assets were reloaded
    pub fn has_changes(&self) -> bool {
        self.textures_reloaded > 0 || self.models_reloaded > 0 || !self.scripts_changed.is_empty()
    }

    /// Check if any errors occurred
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Get total count of reloaded assets
    pub fn total_reloaded(&self) -> usize {
        self.textures_reloaded + self.models_reloaded
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_registry() {
        let mut registry = AssetRegistry::new();

        registry.register_texture(
            PathBuf::from("/assets/test.png"),
            "test.png".to_string(),
        );

        assert_eq!(
            registry.get_texture_path(&PathBuf::from("/assets/test.png")),
            Some("test.png")
        );

        let stats = registry.stats();
        assert_eq!(stats.texture_count, 1);
    }

    #[test]
    fn test_reload_result() {
        let mut result = HotReloadResult::default();
        assert!(!result.has_changes());

        result.textures_reloaded = 1;
        assert!(result.has_changes());
        assert_eq!(result.total_reloaded(), 1);
    }
}
