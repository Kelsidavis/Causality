// Skybox rendering - cubemap environment

use crate::MSAA_SAMPLE_COUNT;
use anyhow::Result;
use wgpu::util::DeviceExt;

/// Skybox configuration
pub const SKYBOX_SIZE: u32 = 1024;

/// Skybox renderer
pub struct Skybox {
    /// Cubemap texture
    pub texture: wgpu::Texture,
    /// Cubemap view
    pub view: wgpu::TextureView,
    /// Cubemap sampler
    pub sampler: wgpu::Sampler,
    /// Bind group
    pub bind_group: wgpu::BindGroup,
    /// Render pipeline
    pub render_pipeline: wgpu::RenderPipeline,
}

impl Skybox {
    /// Create a new skybox
    pub fn new(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        camera_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Result<Self> {
        // Create cubemap texture
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Skybox Cubemap"),
            size: wgpu::Extent3d {
                width: SKYBOX_SIZE,
                height: SKYBOX_SIZE,
                depth_or_array_layers: 6, // 6 faces
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Skybox View"),
            dimension: Some(wgpu::TextureViewDimension::Cube),
            ..Default::default()
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Skybox Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        // Create bind group layout for skybox texture
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Skybox Bind Group Layout"),
            entries: &[
                // Cubemap texture
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::Cube,
                        multisampled: false,
                    },
                    count: None,
                },
                // Sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Skybox Bind Group"),
            layout: &bind_group_layout,
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

        // Create shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Skybox Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/skybox.wgsl").into()),
        });

        // Create pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Skybox Pipeline Layout"),
            bind_group_layouts: &[camera_bind_group_layout, &bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create render pipeline
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Skybox Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None, // Render both sides
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: false, // Don't write to depth
                depth_compare: wgpu::CompareFunction::LessEqual, // Draw at far plane
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: MSAA_SAMPLE_COUNT,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        Ok(Self {
            texture,
            view,
            sampler,
            bind_group,
            render_pipeline,
        })
    }

    /// Load cubemap faces from image data
    /// Order: +X, -X, +Y, -Y, +Z, -Z
    pub fn load_cubemap(&self, queue: &wgpu::Queue, face_data: &[(&[u8], u32, u32)]) -> Result<()> {
        if face_data.len() != 6 {
            return Err(anyhow::anyhow!("Cubemap must have exactly 6 faces"));
        }

        for (face_index, (data, width, height)) in face_data.iter().enumerate() {
            if *width != SKYBOX_SIZE || *height != SKYBOX_SIZE {
                return Err(anyhow::anyhow!(
                    "Cubemap face size must be {}x{}, got {}x{}",
                    SKYBOX_SIZE,
                    SKYBOX_SIZE,
                    width,
                    height
                ));
            }

            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &self.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: 0,
                        y: 0,
                        z: face_index as u32,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                data,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * SKYBOX_SIZE),
                    rows_per_image: Some(SKYBOX_SIZE),
                },
                wgpu::Extent3d {
                    width: SKYBOX_SIZE,
                    height: SKYBOX_SIZE,
                    depth_or_array_layers: 1,
                },
            );
        }

        Ok(())
    }

    /// Create a simple gradient skybox (blue to white)
    pub fn create_gradient_skybox(queue: &wgpu::Queue, texture: &wgpu::Texture) {
        let size = SKYBOX_SIZE as usize;
        let mut data = vec![0u8; size * size * 4];

        for face in 0..6 {
            for y in 0..size {
                for x in 0..size {
                    let idx = (y * size + x) * 4;

                    // Simple gradient from blue (bottom) to white (top)
                    let t = y as f32 / size as f32;
                    let r = (135.0 + (255.0 - 135.0) * t) as u8;
                    let g = (206.0 + (255.0 - 206.0) * t) as u8;
                    let b = (235.0 + (255.0 - 235.0) * t) as u8;

                    data[idx] = r;
                    data[idx + 1] = g;
                    data[idx + 2] = b;
                    data[idx + 3] = 255;
                }
            }

            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: 0,
                        y: 0,
                        z: face as u32,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                &data,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * SKYBOX_SIZE),
                    rows_per_image: Some(SKYBOX_SIZE),
                },
                wgpu::Extent3d {
                    width: SKYBOX_SIZE,
                    height: SKYBOX_SIZE,
                    depth_or_array_layers: 1,
                },
            );
        }
    }
}
