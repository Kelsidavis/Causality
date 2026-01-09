// Engine Assets - Asset loading and management

pub mod hot_reload;
pub mod hot_reload_manager;
pub mod loaders;
pub mod manager;
pub mod material;
pub mod mesh;
pub mod terrain;
pub mod texture;
pub mod water_fill;

pub use hot_reload::{HotReloadWatcher, ReloadEvent};
pub use hot_reload_manager::{AssetRegistry, AssetRegistryStats, HotReloadManager, HotReloadResult};
pub use manager::{AssetHandle, AssetManager};
pub use material::{AlphaMode, Material};
pub use mesh::{Mesh, Vertex};
pub use terrain::{HeightMap, Terrain, TerrainConfig};
pub use texture::{Texture, TextureFormat};
pub use water_fill::{compute_water_fill, generate_water_mesh, ComputedWaterBody, WaterFillResult};
