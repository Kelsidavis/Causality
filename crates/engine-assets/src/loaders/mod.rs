// Asset loaders

pub mod gltf_loader;
pub mod material_loader;

pub use gltf_loader::load_gltf;
pub use material_loader::{load_material, save_material};
