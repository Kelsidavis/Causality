// GPU texture handle and management

use engine_assets::Texture;
use wgpu::util::DeviceExt;

/// Handle to a GPU texture
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureHandle(pub usize);

/// GPU texture with view and sampler
pub struct GpuTexture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub bind_group: wgpu::BindGroup,
}

impl GpuTexture {
    /// Create a GPU texture from CPU texture data
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

        let gpu_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&texture.name),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        // Convert RGB to RGBA if needed
        let data = if texture.format == engine_assets::TextureFormat::Rgb8 {
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

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &gpu_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * texture.width), // Always RGBA after conversion
                rows_per_image: Some(texture.height),
            },
            size,
        );

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
            lod_max_clamp: 100.0,
            compare: None,
            anisotropy_clamp: 1,
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

    /// Create a white 1x1 texture (default/fallback)
    pub fn white_texture(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let texture = Texture::solid_color("White".to_string(), [255, 255, 255, 255]);
        Self::from_cpu_texture(device, queue, &texture, bind_group_layout)
    }
}
