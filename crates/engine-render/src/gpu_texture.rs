// GPU texture handle and management
//!
//! This module provides GPU-side texture management for the Causality Engine.
//! Textures are uploaded to the GPU and made available for rendering through
//! bind groups that can be bound to shader pipelines.
//!
//! # Features
//!
//! - Automatic format conversion (RGB8 → RGBA8)
//! - sRGB color space support for correct color reproduction
//! - Linear filtering with repeat wrapping
//! - Fallback white texture for missing textures
//!
//! # Example
//!
//! ```rust,no_run
//! use engine_assets::Texture;
//! use engine_render::GpuTexture;
//!
//! // Load texture from disk
//! let texture = Texture::from_file("assets/stone.png")?;
//!
//! // Upload to GPU
//! let gpu_texture = GpuTexture::from_cpu_texture(
//!     &device,
//!     &queue,
//!     &texture,
//!     &bind_group_layout
//! );
//!
//! // Use in rendering
//! render_pass.set_bind_group(1, &gpu_texture.bind_group, &[]);
//! ```

use engine_assets::Texture;

/// Handle to a GPU texture.
///
/// Provides type-safe access to textures in the texture manager.
/// Handles are cheap to copy and can be stored in components.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureHandle(pub usize);

/// GPU texture with all resources needed for rendering.
///
/// Contains the GPU texture, view for sampling, sampler configuration,
/// and a pre-built bind group ready to be bound to the pipeline.
///
/// # Fields
///
/// - `texture`: The GPU texture resource
/// - `view`: Texture view for shader sampling
/// - `sampler`: Sampling configuration (linear filtering, repeat wrap)
/// - `bind_group`: Pre-configured bind group for binding to pipeline
pub struct GpuTexture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub bind_group: wgpu::BindGroup,
}

impl GpuTexture {
    /// Create a GPU texture from CPU texture data.
    ///
    /// Uploads texture data to the GPU and creates all necessary resources
    /// for rendering (texture, view, sampler, bind group).
    ///
    /// # Arguments
    ///
    /// - `device`: The GPU device to create resources on
    /// - `queue`: The GPU queue for uploading texture data
    /// - `texture`: The CPU-side texture data to upload
    /// - `bind_group_layout`: The bind group layout for creating the bind group
    ///
    /// # Format Conversion
    ///
    /// - **RGBA8** → `Rgba8UnormSrgb` (direct)
    /// - **RGB8** → `Rgba8UnormSrgb` (adds alpha channel = 255)
    /// - **R8** → `R8Unorm` (grayscale)
    ///
    /// All textures use sRGB color space for correct color reproduction.
    ///
    /// # Sampler Configuration
    ///
    /// - Address mode: Repeat (for tiling)
    /// - Mag/Min filter: Linear (smooth interpolation)
    /// - Mipmap filter: Linear (no mipmaps currently)
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// let texture = Texture::from_file("stone.png")?;
    /// let gpu_texture = GpuTexture::from_cpu_texture(
    ///     &device,
    ///     &queue,
    ///     &texture,
    ///     &layout
    /// );
    /// ```
    pub fn from_cpu_texture(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture: &Texture,
        bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let size = wgpu::Extent3d {
            width: texture.width,
            height: texture.height,
            depth_or_array_layers: 1,
        };

        let format = match texture.format {
            engine_assets::TextureFormat::Rgba8 => wgpu::TextureFormat::Rgba8UnormSrgb,
            engine_assets::TextureFormat::Rgb8 => wgpu::TextureFormat::Rgba8UnormSrgb, // Convert RGB to RGBA
            engine_assets::TextureFormat::R8 => wgpu::TextureFormat::R8Unorm,
        };

        // Calculate mip levels for texture creation
        let mip_level_count = (texture.width.max(texture.height) as f32).log2().floor() as u32 + 1;

        let gpu_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&texture.name),
            size,
            mip_level_count,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        // Convert RGB to RGBA if needed
        let base_data = if texture.format == engine_assets::TextureFormat::Rgb8 {
            let mut rgba_data = Vec::with_capacity(texture.width as usize * texture.height as usize * 4);
            for chunk in texture.data.chunks(3) {
                rgba_data.push(chunk[0]);
                rgba_data.push(chunk[1]);
                rgba_data.push(chunk[2]);
                rgba_data.push(255); // Alpha
            }
            rgba_data
        } else {
            texture.data.clone()
        };

        // Upload base mip level
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &gpu_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &base_data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * texture.width), // Always RGBA after conversion
                rows_per_image: Some(texture.height),
            },
            size,
        );

        // Generate and upload mipmaps
        let mut current_data = base_data;
        let mut current_width = texture.width;
        let mut current_height = texture.height;

        for mip_level in 1..mip_level_count {
            let new_width = (current_width / 2).max(1);
            let new_height = (current_height / 2).max(1);

            // Generate downsampled mip using box filter (2x2 average)
            let mut new_data = Vec::with_capacity((new_width * new_height * 4) as usize);
            for y in 0..new_height {
                for x in 0..new_width {
                    // Sample 2x2 pixels from previous mip level
                    let sx = (x * 2) as usize;
                    let sy = (y * 2) as usize;

                    let mut r = 0u32;
                    let mut g = 0u32;
                    let mut b = 0u32;
                    let mut a = 0u32;
                    let mut count = 0u32;

                    for dy in 0..2 {
                        for dx in 0..2 {
                            let px = (sx + dx).min(current_width as usize - 1);
                            let py = (sy + dy).min(current_height as usize - 1);
                            let idx = (py * current_width as usize + px) * 4;
                            if idx + 3 < current_data.len() {
                                r += current_data[idx] as u32;
                                g += current_data[idx + 1] as u32;
                                b += current_data[idx + 2] as u32;
                                a += current_data[idx + 3] as u32;
                                count += 1;
                            }
                        }
                    }

                    if count > 0 {
                        new_data.push((r / count) as u8);
                        new_data.push((g / count) as u8);
                        new_data.push((b / count) as u8);
                        new_data.push((a / count) as u8);
                    } else {
                        new_data.extend_from_slice(&[255, 255, 255, 255]);
                    }
                }
            }

            // Upload this mip level
            let mip_size = wgpu::Extent3d {
                width: new_width,
                height: new_height,
                depth_or_array_layers: 1,
            };

            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &gpu_texture,
                    mip_level,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &new_data,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * new_width),
                    rows_per_image: Some(new_height),
                },
                mip_size,
            );

            current_data = new_data;
            current_width = new_width;
            current_height = new_height;
        }

        let view = gpu_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some(&format!("{} Sampler", texture.name)),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            lod_min_clamp: 0.0,
            lod_max_clamp: mip_level_count as f32,
            compare: None,
            anisotropy_clamp: 16, // Enable 16x anisotropic filtering
            border_color: None,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("{} Bind Group", texture.name)),
            layout: bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        Self {
            texture: gpu_texture,
            view,
            sampler,
            bind_group,
        }
    }

    /// Create a white 1x1 texture (default/fallback).
    ///
    /// Creates a solid white texture that can be used as a fallback
    /// when a texture is missing or as a default for untextured meshes.
    ///
    /// The white color (255, 255, 255, 255) acts as a neutral multiplier,
    /// allowing vertex colors and lighting to show through without tinting.
    ///
    /// # Arguments
    ///
    /// - `device`: The GPU device to create the texture on
    /// - `queue`: The GPU queue for uploading data
    /// - `bind_group_layout`: The bind group layout
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// let fallback = GpuTexture::white_texture(&device, &queue, &layout);
    /// // Use when texture is missing
    /// let texture = texture_manager.get_texture(handle)
    ///     .unwrap_or(&fallback);
    /// ```
    pub fn white_texture(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let texture = Texture::solid_color("White".to_string(), [255, 255, 255, 255]);
        Self::from_cpu_texture(device, queue, &texture, bind_group_layout)
    }
}
