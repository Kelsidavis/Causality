// Texture manager - handles uploading and managing GPU textures

use crate::gpu_texture::{GpuTexture, TextureHandle};
use engine_assets::Texture;
use std::collections::HashMap;

pub struct TextureManager {
    textures: Vec<GpuTexture>,
    texture_map: HashMap<String, TextureHandle>,
    bind_group_layout: wgpu::BindGroupLayout,
    white_texture_handle: TextureHandle,
}

impl TextureManager {
    /// Create the bind group layout descriptor for textures (static definition)
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

    /// Upload a texture to the GPU and return a handle
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
