// GPU-side mesh representation

use anyhow::Result;
use wgpu::util::DeviceExt;

/// Vertex format for GPU rendering
/// Total size: 80 bytes (aligned to 16 bytes)
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuVertex {
    pub position: [f32; 3],      // Offset 0  (12 bytes)
    pub normal: [f32; 3],         // Offset 12 (12 bytes)
    pub tex_coord: [f32; 2],      // Offset 24 (8 bytes)
    pub color: [f32; 3],          // Offset 32 (12 bytes)
    pub tangent: [f32; 4],        // Offset 44 (16 bytes) - w is handedness
    pub bitangent: [f32; 3],      // Offset 60 (12 bytes)
    pub _padding: [f32; 2],       // Offset 72 (8 bytes) - for 16-byte alignment
}

impl GpuVertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<GpuVertex>() as wgpu::BufferAddress, // 80 bytes
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // Position (location 0)
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // Normal (location 1)
                wgpu::VertexAttribute {
                    offset: 12,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // Tex Coord (location 2)
                wgpu::VertexAttribute {
                    offset: 24,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // Color (location 3)
                wgpu::VertexAttribute {
                    offset: 32,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // Tangent (location 4) - includes handedness in w component
                wgpu::VertexAttribute {
                    offset: 44,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // Bitangent (location 5)
                wgpu::VertexAttribute {
                    offset: 60,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

/// GPU mesh - contains vertex and index buffers
pub struct GpuMesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
}

impl GpuMesh {
    /// Upload a mesh to the GPU
    pub fn from_cpu_mesh(
        device: &wgpu::Device,
        vertices: &[GpuVertex],
        indices: &[u32],
    ) -> Self {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            vertex_buffer,
            index_buffer,
            num_indices: indices.len() as u32,
        }
    }
}

/// Mesh handle - reference to a GPU mesh
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MeshHandle(pub usize);
