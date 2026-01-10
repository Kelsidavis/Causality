// Foliage Renderer - Instanced mesh rendering for vegetation
//
// Renders many instances of the same mesh efficiently using GPU instancing.

use crate::gpu_mesh::GpuMesh;
use crate::MSAA_SAMPLE_COUNT;
use anyhow::Result;
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use wgpu::util::DeviceExt;

/// Per-instance foliage data sent to GPU
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct FoliageInstanceGpu {
    /// Model matrix for this instance (4x4 = 16 floats)
    pub model: [[f32; 4]; 4],
    /// Color tint (RGB + padding)
    pub color_tint: [f32; 4],
}

impl FoliageInstanceGpu {
    pub fn new(position: Vec3, rotation_y: f32, scale: f32, color_tint: Vec3) -> Self {
        // Build model matrix: scale -> rotate Y -> translate
        let scale_mat = Mat4::from_scale(Vec3::splat(scale));
        let rotation_mat = Mat4::from_rotation_y(rotation_y);
        let translation_mat = Mat4::from_translation(position);
        let model = translation_mat * rotation_mat * scale_mat;

        Self {
            model: model.to_cols_array_2d(),
            color_tint: [color_tint.x, color_tint.y, color_tint.z, 1.0],
        }
    }
}

/// Camera uniforms for foliage rendering
#[repr(C, align(16))]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct FoliageCameraUniforms {
    pub view_proj: [[f32; 4]; 4],
    pub camera_pos: [f32; 3],
    pub _padding: f32,
}

/// Foliage renderer for instanced vegetation
pub struct FoliageRenderer {
    /// Render pipeline
    pipeline: wgpu::RenderPipeline,
    /// Camera uniform buffer
    camera_buffer: wgpu::Buffer,
    /// Camera bind group
    camera_bind_group: wgpu::BindGroup,
    /// Instance buffer (resized as needed)
    instance_buffer: Option<wgpu::Buffer>,
    /// Current instance buffer capacity
    instance_capacity: usize,
}

impl FoliageRenderer {
    /// Create a new foliage renderer
    pub fn new(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
    ) -> Result<Self> {
        // Create camera uniform buffer
        let camera_uniforms = FoliageCameraUniforms {
            view_proj: Mat4::IDENTITY.to_cols_array_2d(),
            camera_pos: [0.0; 3],
            _padding: 0.0,
        };

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Foliage Camera Uniform Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create camera bind group layout
        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Foliage Camera Bind Group Layout"),
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
            label: Some("Foliage Camera Bind Group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        // Load shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Foliage Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/foliage.wgsl").into()),
        });

        // Create pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Foliage Pipeline Layout"),
            bind_group_layouts: &[&camera_bind_group_layout],
            push_constant_ranges: &[],
        });

        // Vertex buffer layout for mesh vertices
        let vertex_buffer_layout = wgpu::VertexBufferLayout {
            array_stride: 80, // GpuVertex size
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // Position
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // Normal
                wgpu::VertexAttribute {
                    offset: 12,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // TexCoord
                wgpu::VertexAttribute {
                    offset: 24,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // Color
                wgpu::VertexAttribute {
                    offset: 32,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        };

        // Instance buffer layout
        let instance_buffer_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<FoliageInstanceGpu>() as u64,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                // Model matrix column 0
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // Model matrix column 1
                wgpu::VertexAttribute {
                    offset: 16,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // Model matrix column 2
                wgpu::VertexAttribute {
                    offset: 32,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // Model matrix column 3
                wgpu::VertexAttribute {
                    offset: 48,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // Color tint
                wgpu::VertexAttribute {
                    offset: 64,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        };

        // Create render pipeline
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Foliage Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[vertex_buffer_layout, instance_buffer_layout],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back), // Enable backface culling to reduce flickering
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState {
                    constant: 1,  // Small depth bias to prevent Z-fighting
                    slope_scale: 1.0,
                    clamp: 0.0,
                },
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
            pipeline,
            camera_buffer,
            camera_bind_group,
            instance_buffer: None,
            instance_capacity: 0,
        })
    }

    /// Update camera uniforms
    pub fn update_camera(
        &self,
        queue: &wgpu::Queue,
        view_proj: Mat4,
        camera_pos: Vec3,
    ) {
        let uniforms = FoliageCameraUniforms {
            view_proj: view_proj.to_cols_array_2d(),
            camera_pos: camera_pos.to_array(),
            _padding: 0.0,
        };

        queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[uniforms]));
    }

    /// Update instance buffer with new data
    pub fn update_instances(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        instances: &[FoliageInstanceGpu],
    ) {
        if instances.is_empty() {
            return;
        }

        let required_size = instances.len();

        // Recreate buffer if needed
        if self.instance_buffer.is_none() || self.instance_capacity < required_size {
            // Round up to next power of 2 for less frequent reallocations
            let new_capacity = required_size.next_power_of_two().max(64);

            self.instance_buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Foliage Instance Buffer"),
                size: (new_capacity * std::mem::size_of::<FoliageInstanceGpu>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
            self.instance_capacity = new_capacity;
        }

        // Write instance data
        if let Some(buffer) = &self.instance_buffer {
            queue.write_buffer(buffer, 0, bytemuck::cast_slice(instances));
        }
    }

    /// Render foliage instances with the given mesh
    pub fn render<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        mesh: &'a GpuMesh,
        instance_count: u32,
    ) {
        if instance_count == 0 {
            return;
        }

        let Some(instance_buffer) = &self.instance_buffer else {
            return;
        };

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
        render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
        render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

        render_pass.draw_indexed(0..mesh.num_indices, 0, 0..instance_count);
    }
}

/// Collected foliage data for rendering
pub struct FoliageRenderData {
    /// Mesh name to use
    pub mesh_name: String,
    /// All instances for this mesh type
    pub instances: Vec<FoliageInstanceGpu>,
}

impl FoliageRenderData {
    pub fn new(mesh_name: String) -> Self {
        Self {
            mesh_name,
            instances: Vec::new(),
        }
    }

    pub fn add_instance(&mut self, position: Vec3, rotation_y: f32, scale: f32, color_tint: Vec3) {
        self.instances.push(FoliageInstanceGpu::new(position, rotation_y, scale, color_tint));
    }
}
