// Texture manager - handles uploading and managing GPU textures
//!
//! The texture manager provides centralized texture loading, caching, and GPU upload.
//! It prevents duplicate uploads and provides handle-based access to textures.
//!
//! # Features
//!
//! - **Texture caching** - Prevents duplicate GPU uploads
//! - **Handle-based access** - Type-safe texture references
//! - **Automatic fallback** - White texture for missing textures
//! - **Name-based lookup** - Access textures by string name
//!
//! # Example
//!
//! ```rust,no_run
//! use engine_render::TextureManager;
//! use engine_assets::Texture;
//!
//! // Create manager
//! let mut texture_manager = TextureManager::new(&device, &queue);
//!
//! // Load and upload texture
//! let texture = Texture::from_file("stone.png")?;
//! let handle = texture_manager.upload_texture(
//!     &device,
//!     &queue,
//!     "stone".to_string(),
//!     &texture
//! );
//!
//! // Get texture for rendering
//! let gpu_texture = texture_manager.get_texture(handle).unwrap();
//! render_pass.set_bind_group(1, &gpu_texture.bind_group, &[]);
//! ```

use crate::gpu_texture::{GpuTexture, TextureHandle};
use engine_assets::Texture;
use std::collections::HashMap;

/// Manages texture loading, caching, and GPU upload.
///
/// Provides centralized texture management with automatic caching
/// to prevent duplicate uploads. Textures are accessed via handles
/// and can be looked up by name.
///
/// # Fields
///
/// - `textures`: Vector of GPU textures
/// - `texture_map`: Name to handle mapping for lookup
/// - `bind_group_layout`: Shared bind group layout for all textures
/// - `white_texture_handle`: Handle to default white fallback texture
pub struct TextureManager {
    textures: Vec<GpuTexture>,
    texture_map: HashMap<String, TextureHandle>,
    bind_group_layout: wgpu::BindGroupLayout,
    white_texture_handle: TextureHandle,
}

impl TextureManager {
    /// Create the bind group layout descriptor for textures (static definition).
    ///
    /// Returns a static descriptor that can be used to create bind group layouts
    /// in both the renderer and texture manager, ensuring compatibility.
    ///
    /// The layout defines two bindings for textures:
    /// - Binding 0: Texture view (fragment shader)
    /// - Binding 1: Sampler (fragment shader)
    ///
    /// Both bindings are visible only to the fragment shader stage.
    pub fn bind_group_layout_descriptor() -> wgpu::BindGroupLayoutDescriptor<'static> {
        wgpu::BindGroupLayoutDescriptor {
            label: Some("Texture Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        }
    }

    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        // Create bind group layout for textures
        let bind_group_layout = device.create_bind_group_layout(&Self::bind_group_layout_descriptor());

        // Create default white texture
        let white_texture = GpuTexture::white_texture(device, queue, &bind_group_layout);
        let mut textures = Vec::new();
        textures.push(white_texture);

        let white_handle = TextureHandle(0);
        let mut texture_map = HashMap::new();
        texture_map.insert("white".to_string(), white_handle);

        Self {
            textures,
            texture_map,
            bind_group_layout,
            white_texture_handle: white_handle,
        }
    }

    /// Upload a texture to the GPU and return a handle.
    ///
    /// Uploads texture data to the GPU and returns a handle for accessing it.
    /// If a texture with the same name already exists, returns the existing handle
    /// without uploading again (caching).
    ///
    /// # Arguments
    ///
    /// - `device`: The GPU device to create resources on
    /// - `queue`: The GPU queue for uploading data
    /// - `name`: Unique name for the texture (used for caching and lookup)
    /// - `texture`: The CPU-side texture data to upload
    ///
    /// # Returns
    ///
    /// A `TextureHandle` that can be used to retrieve the GPU texture.
    ///
    /// # Caching
    ///
    /// ```rust,no_run
    /// // First call: uploads to GPU
    /// let handle1 = texture_manager.upload_texture(
    ///     &device, &queue, "stone".to_string(), &texture
    /// );
    ///
    /// // Second call: returns cached handle (no upload)
    /// let handle2 = texture_manager.upload_texture(
    ///     &device, &queue, "stone".to_string(), &texture
    /// );
    ///
    /// assert_eq!(handle1, handle2);
    /// ```
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// let texture = Texture::from_file("assets/stone.png")?;
    /// let handle = texture_manager.upload_texture(
    ///     &device,
    ///     &queue,
    ///     "stone_wall".to_string(),
    ///     &texture
    /// );
    /// ```
    pub fn upload_texture(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        name: String,
        texture: &Texture,
    ) -> TextureHandle {
        // Check if already uploaded
        if let Some(&handle) = self.texture_map.get(&name) {
            return handle;
        }

        let gpu_texture = GpuTexture::from_cpu_texture(device, queue, texture, &self.bind_group_layout);
        let handle = TextureHandle(self.textures.len());
        self.textures.push(gpu_texture);
        self.texture_map.insert(name, handle);

        handle
    }

    /// Get a texture by handle
    pub fn get_texture(&self, handle: TextureHandle) -> Option<&GpuTexture> {
        self.textures.get(handle.0)
    }

    /// Get a texture handle by name
    pub fn get_handle(&self, name: &str) -> Option<TextureHandle> {
        self.texture_map.get(name).copied()
    }

    /// Get the white texture handle (default/fallback)
    pub fn white_texture_handle(&self) -> TextureHandle {
        self.white_texture_handle
    }

    /// Get the bind group layout for textures
    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    /// Get texture count
    pub fn texture_count(&self) -> usize {
        self.textures.len()
    }
}
