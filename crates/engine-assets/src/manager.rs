// Asset Manager - handles loading and caching of assets

use crate::loaders::gltf_loader;
use crate::mesh::Mesh;
use crate::texture::Texture;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Asset handle - cheap to clone, points to cached asset
#[derive(Debug, Clone)]
pub struct AssetHandle<T> {
    pub inner: Arc<T>,
}

impl<T> AssetHandle<T> {
    pub fn new(asset: T) -> Self {
        Self {
            inner: Arc::new(asset),
        }
    }

    pub fn get(&self) -> &T {
        &self.inner
    }
}

pub struct AssetManager {
    asset_root: PathBuf,
    meshes: HashMap<PathBuf, AssetHandle<Vec<Mesh>>>,
    textures: HashMap<PathBuf, AssetHandle<Texture>>,
}

impl AssetManager {
    pub fn new<P: AsRef<Path>>(asset_root: P) -> Self {
        Self {
            asset_root: asset_root.as_ref().to_path_buf(),
            meshes: HashMap::new(),
            textures: HashMap::new(),
        }
    }

    /// Get the full path for an asset
    fn full_path(&self, relative_path: &str) -> PathBuf {
        self.asset_root.join(relative_path)
    }

    /// Load a GLTF model (with caching)
    pub fn load_gltf(&mut self, path: &str) -> Result<AssetHandle<Vec<Mesh>>> {
        let full_path = self.full_path(path);

        // Check cache
        if let Some(handle) = self.meshes.get(&full_path) {
            return Ok(handle.clone());
        }

        // Load from disk
        log::info!("Loading GLTF: {:?}", full_path);
        let meshes = gltf_loader::load_gltf(&full_path)
            .with_context(|| format!("Failed to load GLTF: {}", path))?;

        let handle = AssetHandle::new(meshes);
        self.meshes.insert(full_path, handle.clone());

        Ok(handle)
    }

    /// Load a texture (with caching)
    pub fn load_texture(&mut self, path: &str) -> Result<AssetHandle<Texture>> {
        let full_path = self.full_path(path);

        // Check cache
        if let Some(handle) = self.textures.get(&full_path) {
            return Ok(handle.clone());
        }

        // Load from disk
        log::info!("Loading texture: {:?}", full_path);
        let texture = Texture::from_file(&full_path)
            .with_context(|| format!("Failed to load texture: {}", path))?;

        let handle = AssetHandle::new(texture);
        self.textures.insert(full_path, handle.clone());

        Ok(handle)
    }

    /// Create a mesh directly (and cache it)
    pub fn create_mesh(&mut self, name: String, mesh: Mesh) -> AssetHandle<Vec<Mesh>> {
        let path = PathBuf::from(format!("__generated__/{}", name));
        let handle = AssetHandle::new(vec![mesh]);
        self.meshes.insert(path, handle.clone());
        handle
    }

    /// Get cached mesh count
    pub fn mesh_cache_size(&self) -> usize {
        self.meshes.len()
    }

    /// Get cached texture count
    pub fn texture_cache_size(&self) -> usize {
        self.textures.len()
    }

    /// Clear all caches
    pub fn clear_cache(&mut self) {
        self.meshes.clear();
        self.textures.clear();
        log::info!("Asset cache cleared");
    }
}

impl Default for AssetManager {
    fn default() -> Self {
        Self::new("assets")
    }
}
