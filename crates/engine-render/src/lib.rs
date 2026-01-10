// Engine Render - 3D rendering with wgpu

/// MSAA sample count for anti-aliasing (1 = off, 4 = 4x MSAA)
pub const MSAA_SAMPLE_COUNT: u32 = 4;

pub mod camera;
pub mod culling;
pub mod frustum;
pub mod gpu_material;
pub mod gpu_mesh;
pub mod gpu_texture;
pub mod lod;
pub mod material_manager;
pub mod mesh_manager;
pub mod particle_renderer;
pub mod postprocess;
pub mod renderer;
pub mod shadow;
pub mod skybox;
pub mod texture_manager;
pub mod water;

pub use camera::Camera;
pub use culling::{CullingStats, CullingSystem, Renderable, RenderableId, VisibilityResult};
pub use frustum::{Frustum, Plane, AABB};
pub use gpu_material::{GpuMaterial, MaterialHandle, MaterialUniforms};
pub use gpu_mesh::{GpuMesh, GpuVertex, MeshHandle};
pub use gpu_texture::{GpuTexture, TextureHandle};
pub use lod::{distance_squared, LodBias, LodConfig, LodLevel};
pub use material_manager::MaterialManager;
pub use mesh_manager::MeshManager;
pub use particle_renderer::{ParticleBlendMode, ParticleCameraUniforms, ParticleRenderer};
pub use postprocess::{CompositePushConstants, Framebuffer, PostProcessPipeline, PostProcessSettings};
pub use renderer::Renderer;
pub use shadow::{ShadowMap, ShadowUniforms, ShadowPushConstants};
pub use skybox::Skybox;
pub use texture_manager::TextureManager;
pub use water::{WaterRenderer, WaterUniforms, WaterPushConstants};
