// Material manager - handles uploading and managing GPU materials

use crate::gpu_material::{GpuMaterial, MaterialHandle, MaterialUniforms};
use crate::gpu_texture::TextureHandle;
use crate::texture_manager::TextureManager;
use engine_assets::Material;
use std::collections::HashMap;
use wgpu::util::DeviceExt;

/// Manages material uploading, caching, and GPU resources
pub struct MaterialManager {
    materials: Vec<GpuMaterial>,
    material_map: HashMap<String, MaterialHandle>,
    bind_group_layout: wgpu::BindGroupLayout,
    default_material_handle: MaterialHandle,
}

impl MaterialManager {
    /// Create a new material manager with a default material
    pub fn new(
        device: &wgpu::Device,
        texture_manager: &TextureManager,
    ) -> Self {
        // Create bind group layout
        let bind_group_layout = GpuMaterial::create_bind_group_layout(device);

        // Create default material (white, non-metallic, medium roughness)
        let default_material = Material::default();
        let white_handle = texture_manager.white_texture_handle();

        let gpu_material = Self::create_gpu_material(
            device,
            texture_manager,
            &bind_group_layout,
            "default".to_string(),
            &default_material,
            white_handle,
            None,
            None,
            None,
        );

        let mut materials = Vec::new();
        materials.push(gpu_material);

        let default_handle = MaterialHandle(0);
        let mut material_map = HashMap::new();
        material_map.insert("default".to_string(), default_handle);

        Self {
            materials,
            material_map,
            bind_group_layout,
            default_material_handle: default_handle,
        }
    }

    /// Upload a material to the GPU
    ///
    /// Takes a CPU-side material and texture handles, uploads the material parameters
    /// to a GPU uniform buffer, and creates a bind group with all textures.
    ///
    /// # Arguments
    ///
    /// - `device`: GPU device
    /// - `texture_manager`: Texture manager for accessing GPU textures
    /// - `name`: Unique name for caching
    /// - `material`: CPU-side material definition
    /// - `albedo_handle`: Handle to albedo/base color texture
    /// - `normal_handle`: Optional handle to normal map
    /// - `metallic_roughness_handle`: Optional handle to metallic/roughness texture
    /// - `ao_handle`: Optional handle to ambient occlusion texture
    ///
    /// # Returns
    ///
    /// A `MaterialHandle` that can be used to retrieve the GPU material
    pub fn upload_material(
        &mut self,
        device: &wgpu::Device,
        texture_manager: &TextureManager,
        name: String,
        material: &Material,
        albedo_handle: TextureHandle,
        normal_handle: Option<TextureHandle>,
        metallic_roughness_handle: Option<TextureHandle>,
        ao_handle: Option<TextureHandle>,
    ) -> MaterialHandle {
        // Check if already uploaded
        if let Some(&handle) = self.material_map.get(&name) {
            return handle;
        }

        let gpu_material = Self::create_gpu_material(
            device,
            texture_manager,
            &self.bind_group_layout,
            name.clone(),
            material,
            albedo_handle,
            normal_handle,
            metallic_roughness_handle,
            ao_handle,
        );

        let handle = MaterialHandle(self.materials.len());
        self.materials.push(gpu_material);
        self.material_map.insert(name, handle);

        handle
    }

    /// Internal: Create a GPU material with bind group
    fn create_gpu_material(
        device: &wgpu::Device,
        texture_manager: &TextureManager,
        bind_group_layout: &wgpu::BindGroupLayout,
        name: String,
        material: &Material,
        albedo_handle: TextureHandle,
        normal_handle: Option<TextureHandle>,
        metallic_roughness_handle: Option<TextureHandle>,
        ao_handle: Option<TextureHandle>,
    ) -> GpuMaterial {
        // Create material uniforms
        let uniforms = MaterialUniforms::from_material(material);

        // Create uniform buffer
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("Material Uniform Buffer: {}", name)),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Get textures (use white as fallback)
        let white_handle = texture_manager.white_texture_handle();
        let albedo_texture = texture_manager.get_texture(albedo_handle).unwrap();
        let normal_texture = texture_manager.get_texture(normal_handle.unwrap_or(white_handle)).unwrap();
        let metallic_roughness_texture = texture_manager.get_texture(metallic_roughness_handle.unwrap_or(white_handle)).unwrap();
        let ao_texture = texture_manager.get_texture(ao_handle.unwrap_or(white_handle)).unwrap();

        // Create bind group with all textures and uniforms
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("Material Bind Group: {}", name)),
            layout: bind_group_layout,
            entries: &[
                // Binding 0: Material uniforms
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                // Binding 1: Albedo texture
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&albedo_texture.view),
                },
                // Binding 2: Albedo sampler
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&albedo_texture.sampler),
                },
                // Binding 3: Normal texture
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&normal_texture.view),
                },
                // Binding 4: Normal sampler
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::Sampler(&normal_texture.sampler),
                },
                // Binding 5: Metallic-roughness texture
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::TextureView(&metallic_roughness_texture.view),
                },
                // Binding 6: Metallic-roughness sampler
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: wgpu::BindingResource::Sampler(&metallic_roughness_texture.sampler),
                },
                // Binding 7: AO texture
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: wgpu::BindingResource::TextureView(&ao_texture.view),
                },
                // Binding 8: AO sampler
                wgpu::BindGroupEntry {
                    binding: 8,
                    resource: wgpu::BindingResource::Sampler(&ao_texture.sampler),
                },
            ],
        });

        GpuMaterial {
            name,
            albedo_texture: albedo_handle,
            normal_texture: normal_handle,
            metallic_roughness_texture: metallic_roughness_handle,
            ao_texture: ao_handle,
            uniforms,
            uniform_buffer,
            bind_group,
            alpha_mode: material.alpha_mode,
            double_sided: material.double_sided,
        }
    }

    /// Get a material by handle
    pub fn get_material(&self, handle: MaterialHandle) -> Option<&GpuMaterial> {
        self.materials.get(handle.0)
    }

    /// Get a material handle by name
    pub fn get_handle(&self, name: &str) -> Option<MaterialHandle> {
        self.material_map.get(name).copied()
    }

    /// Get the default material handle
    pub fn default_material_handle(&self) -> MaterialHandle {
        self.default_material_handle
    }

    /// Get the bind group layout for materials
    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    /// Get material count
    pub fn material_count(&self) -> usize {
        self.materials.len()
    }
}
