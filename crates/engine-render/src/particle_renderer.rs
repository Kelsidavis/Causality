// Particle Renderer - Instanced billboard rendering

use anyhow::Result;
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use wgpu::util::DeviceExt;

/// Camera uniforms for particle rendering
#[repr(C, align(16))]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct ParticleCameraUniforms {
    pub view_proj: [[f32; 4]; 4],
    pub camera_pos: [f32; 3],
    pub _padding1: f32,
    pub camera_right: [f32; 3],
    pub _padding2: f32,
    pub camera_up: [f32; 3],
    pub _padding3: f32,
}

/// Blend mode for particle rendering
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticleBlendMode {
    Alpha,      // Standard alpha blending
    Additive,   // Additive blending (fire, sparks)
    Multiply,   // Multiplicative blending (smoke)
}

/// Particle renderer for instanced billboard rendering
pub struct ParticleRenderer {
    /// Render pipeline
    pipeline: wgpu::RenderPipeline,

    /// Camera bind group layout
    camera_bind_group_layout: wgpu::BindGroupLayout,

    /// Texture bind group layout
    texture_bind_group_layout: wgpu::BindGroupLayout,

    /// Camera uniform buffer
    camera_buffer: wgpu::Buffer,

    /// Camera bind group
    camera_bind_group: wgpu::BindGroup,

    /// Default texture bind group (white square)
    default_texture_bind_group: wgpu::BindGroup,

    /// Current blend mode
    blend_mode: ParticleBlendMode,
}

impl ParticleRenderer {
    /// Create a new particle renderer
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_format: wgpu::TextureFormat,
        blend_mode: ParticleBlendMode,
    ) -> Result<Self> {
        // Create camera uniform buffer
        let camera_uniforms = ParticleCameraUniforms {
            view_proj: Mat4::IDENTITY.to_cols_array_2d(),
            camera_pos: [0.0; 3],
            _padding1: 0.0,
            camera_right: [1.0, 0.0, 0.0],
            _padding2: 0.0,
            camera_up: [0.0, 1.0, 0.0],
            _padding3: 0.0,
        };

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Particle Camera Uniform Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create camera bind group layout
        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Particle Camera Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        // Create camera bind group
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Particle Camera Bind Group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        // Create texture bind group layout
        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Particle Texture Bind Group Layout"),
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
            });

        // Create default white texture (1x1 white pixel)
        let white_pixel = [255u8, 255, 255, 255];
        let default_texture = device.create_texture_with_data(
            queue,
            &wgpu::TextureDescriptor {
                label: Some("Default Particle Texture"),
                size: wgpu::Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            &white_pixel,
        );

        let default_texture_view = default_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let default_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Default Particle Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let default_texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Default Particle Texture Bind Group"),
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&default_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&default_sampler),
                },
            ],
        });

        // Load shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Particle Render Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/particle_render.wgsl").into()),
        });

        // Create pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Particle Render Pipeline Layout"),
            bind_group_layouts: &[&camera_bind_group_layout, &texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        // Define blend state based on mode
        let blend_state = match blend_mode {
            ParticleBlendMode::Alpha => wgpu::BlendState::ALPHA_BLENDING,
            ParticleBlendMode::Additive => wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::SrcAlpha,
                    dst_factor: wgpu::BlendFactor::One,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha: wgpu::BlendComponent::OVER,
            },
            ParticleBlendMode::Multiply => wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::Dst,
                    dst_factor: wgpu::BlendFactor::Zero,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha: wgpu::BlendComponent::OVER,
            },
        };

        // Vertex buffer layout (per-instance particle data)
        let vertex_buffer_layout = wgpu::VertexBufferLayout {
            array_stride: 64, // GpuParticle size
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                // Position (location 0)
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // Velocity (location 1) - skip padding
                wgpu::VertexAttribute {
                    offset: 16,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // Color (location 2)
                wgpu::VertexAttribute {
                    offset: 32,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // Size (location 3)
                wgpu::VertexAttribute {
                    offset: 48,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32,
                },
                // Lifetime (location 4)
                wgpu::VertexAttribute {
                    offset: 52,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32,
                },
                // Max lifetime (location 5)
                wgpu::VertexAttribute {
                    offset: 56,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32,
                },
                // Rotation (location 6)
                wgpu::VertexAttribute {
                    offset: 60,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32,
                },
            ],
        };

        // Create render pipeline
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Particle Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[vertex_buffer_layout],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(blend_state),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None, // No culling for billboards
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: false, // Don't write depth (transparent)
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Ok(Self {
            pipeline,
            camera_bind_group_layout,
            texture_bind_group_layout,
            camera_buffer,
            camera_bind_group,
            default_texture_bind_group,
            blend_mode,
        })
    }

    /// Update camera uniforms
    pub fn update_camera(
        &self,
        queue: &wgpu::Queue,
        view_proj: Mat4,
        camera_pos: Vec3,
        camera_right: Vec3,
        camera_up: Vec3,
    ) {
        let uniforms = ParticleCameraUniforms {
            view_proj: view_proj.to_cols_array_2d(),
            camera_pos: camera_pos.to_array(),
            _padding1: 0.0,
            camera_right: camera_right.to_array(),
            _padding2: 0.0,
            camera_up: camera_up.to_array(),
            _padding3: 0.0,
        };

        queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[uniforms]));
    }

    /// Render particles
    pub fn render<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        particle_buffer: &'a wgpu::Buffer,
        particle_count: u32,
    ) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
        render_pass.set_bind_group(1, &self.default_texture_bind_group, &[]);
        render_pass.set_vertex_buffer(0, particle_buffer.slice(..));

        // Draw 6 vertices (2 triangles) per particle instance
        render_pass.draw(0..6, 0..particle_count);
    }

    /// Get blend mode
    pub fn blend_mode(&self) -> ParticleBlendMode {
        self.blend_mode
    }
}
