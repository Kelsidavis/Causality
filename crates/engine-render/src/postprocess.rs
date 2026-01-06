// Post-processing pipeline - framebuffer effects

use anyhow::Result;
use wgpu::util::DeviceExt;

/// Framebuffer for post-processing
pub struct Framebuffer {
    /// Color texture
    pub texture: wgpu::Texture,
    /// Color texture view
    pub view: wgpu::TextureView,
    /// Depth texture (optional)
    pub depth_texture: Option<wgpu::Texture>,
    /// Depth texture view (optional)
    pub depth_view: Option<wgpu::TextureView>,
    /// Sampler
    pub sampler: wgpu::Sampler,
}

impl Framebuffer {
    /// Create a new framebuffer
    pub fn new(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
        with_depth: bool,
    ) -> Result<Self> {
        // Create color texture
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Framebuffer Color"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Create depth texture if requested
        let (depth_texture, depth_view) = if with_depth {
            let depth_tex = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Framebuffer Depth"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth32Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });

            let depth_v = depth_tex.create_view(&wgpu::TextureViewDescriptor::default());
            (Some(depth_tex), Some(depth_v))
        } else {
            (None, None)
        };

        // Create sampler
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Framebuffer Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Ok(Self {
            texture,
            view,
            depth_texture,
            depth_view,
            sampler,
        })
    }
}

/// Post-processing pipeline
pub struct PostProcessPipeline {
    /// Bind group layout for source texture
    pub bind_group_layout: wgpu::BindGroupLayout,
    /// Tone mapping pipeline
    pub tonemap_pipeline: wgpu::RenderPipeline,
    /// Bloom bright pass pipeline
    pub bloom_bright_pipeline: wgpu::RenderPipeline,
    /// Bloom blur pipeline (horizontal)
    pub bloom_blur_h_pipeline: wgpu::RenderPipeline,
    /// Bloom blur pipeline (vertical)
    pub bloom_blur_v_pipeline: wgpu::RenderPipeline,
    /// Final composite pipeline
    pub composite_pipeline: wgpu::RenderPipeline,
}

impl PostProcessPipeline {
    /// Create a new post-processing pipeline
    pub fn new(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
    ) -> Result<Self> {
        // Create bind group layout for texture sampling
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Post-Process Bind Group Layout"),
            entries: &[
                // Texture
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
                // Sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        // Create shaders
        let tonemap_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Tonemap Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/tonemap.wgsl").into()),
        });

        let bloom_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Bloom Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/bloom.wgsl").into()),
        });

        let composite_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Composite Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/composite.wgsl").into()),
        });

        // Pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Post-Process Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create pipelines
        let tonemap_pipeline = Self::create_fullscreen_pipeline(
            device,
            &pipeline_layout,
            &tonemap_shader,
            surface_format,
            "Tonemap Pipeline",
        );

        let bloom_bright_pipeline = Self::create_fullscreen_pipeline(
            device,
            &pipeline_layout,
            &bloom_shader,
            wgpu::TextureFormat::Rgba16Float,
            "Bloom Bright Pipeline",
        );

        let bloom_blur_h_pipeline = Self::create_fullscreen_pipeline(
            device,
            &pipeline_layout,
            &bloom_shader,
            wgpu::TextureFormat::Rgba16Float,
            "Bloom Blur H Pipeline",
        );

        let bloom_blur_v_pipeline = Self::create_fullscreen_pipeline(
            device,
            &pipeline_layout,
            &bloom_shader,
            wgpu::TextureFormat::Rgba16Float,
            "Bloom Blur V Pipeline",
        );

        let composite_pipeline = Self::create_fullscreen_pipeline(
            device,
            &pipeline_layout,
            &composite_shader,
            surface_format,
            "Composite Pipeline",
        );

        Ok(Self {
            bind_group_layout,
            tonemap_pipeline,
            bloom_bright_pipeline,
            bloom_blur_h_pipeline,
            bloom_blur_v_pipeline,
            composite_pipeline,
        })
    }

    /// Create a fullscreen quad pipeline
    fn create_fullscreen_pipeline(
        device: &wgpu::Device,
        layout: &wgpu::PipelineLayout,
        shader: &wgpu::ShaderModule,
        format: wgpu::TextureFormat,
        label: &str,
    ) -> wgpu::RenderPipeline {
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(label),
            layout: Some(layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        })
    }

    /// Create bind group for a texture
    pub fn create_bind_group(
        &self,
        device: &wgpu::Device,
        view: &wgpu::TextureView,
        sampler: &wgpu::Sampler,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Post-Process Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ],
        })
    }
}

/// Post-processing settings
#[derive(Debug, Clone)]
pub struct PostProcessSettings {
    /// Enable tone mapping
    pub enable_tonemap: bool,
    /// Tone mapping exposure
    pub exposure: f32,
    /// Enable bloom
    pub enable_bloom: bool,
    /// Bloom threshold
    pub bloom_threshold: f32,
    /// Bloom intensity
    pub bloom_intensity: f32,
}

impl Default for PostProcessSettings {
    fn default() -> Self {
        Self {
            enable_tonemap: true,
            exposure: 1.0,
            enable_bloom: false,
            bloom_threshold: 1.0,
            bloom_intensity: 0.3,
        }
    }
}
