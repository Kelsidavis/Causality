// GPU material representation with PBR textures and parameters

use crate::gpu_texture::TextureHandle;
use engine_assets::{AlphaMode, Material};
use wgpu::util::DeviceExt;

/// Handle to a GPU material
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MaterialHandle(pub usize);

/// Material uniforms uploaded to GPU
/// Must match WGSL struct140 layout rules (vec3 aligned to 16 bytes)
#[repr(C, align(16))]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MaterialUniforms {
    pub base_color: [f32; 4],          // Offset 0, size 16
    pub emissive_color: [f32; 3],      // Offset 16, size 12
    pub _padding1: f32,                // Offset 28, size 4 (align vec3 to 16)
    pub metallic: f32,                 // Offset 32
    pub roughness: f32,                // Offset 36
    pub ao_factor: f32,                // Offset 40
    pub emissive_strength: f32,        // Offset 44
    pub alpha_cutoff: f32,             // Offset 48
    pub _padding2: [f32; 3],           // Offset 52, total 64 bytes
}

impl MaterialUniforms {
    pub fn from_material(material: &Material) -> Self {
        Self {
            base_color: material.base_color,
            emissive_color: material.emissive_color,
            _padding1: 0.0,
            metallic: material.metallic,
            roughness: material.roughness,
            ao_factor: material.ao_factor,
            emissive_strength: material.emissive_strength,
            alpha_cutoff: material.alpha_cutoff,
            _padding2: [0.0; 3],
        }
    }
}

/// GPU material with textures and parameters
pub struct GpuMaterial {
    pub name: String,

    // Texture handles
    pub albedo_texture: TextureHandle,
    pub normal_texture: Option<TextureHandle>,
    pub metallic_roughness_texture: Option<TextureHandle>,
    pub ao_texture: Option<TextureHandle>,

    // Material parameters
    pub uniforms: MaterialUniforms,
    pub uniform_buffer: wgpu::Buffer,

    // Bind group with all textures and uniforms
    pub bind_group: wgpu::BindGroup,

    // Rendering properties
    pub alpha_mode: AlphaMode,
    pub double_sided: bool,
}

impl GpuMaterial {
    /// Create bind group layout for materials
    ///
    /// Layout:
    /// - Binding 0: Material uniforms (uniform buffer)
    /// - Binding 1: Albedo texture
    /// - Binding 2: Albedo sampler
    /// - Binding 3: Normal texture
    /// - Binding 4: Normal sampler
    /// - Binding 5: Metallic-roughness texture
    /// - Binding 6: Metallic-roughness sampler
    /// - Binding 7: AO texture
    /// - Binding 8: AO sampler
    pub fn create_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Material Bind Group Layout"),
            entries: &[
                // Binding 0: Material uniforms
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Binding 1: Albedo texture
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // Binding 2: Albedo sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                // Binding 3: Normal texture
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // Binding 4: Normal sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                // Binding 5: Metallic-roughness texture
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // Binding 6: Metallic-roughness sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 6,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                // Binding 7: AO texture
                wgpu::BindGroupLayoutEntry {
                    binding: 7,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // Binding 8: AO sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 8,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        })
    }
}
