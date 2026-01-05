// Mesh manager - handles uploading and managing GPU meshes

use crate::gpu_mesh::{GpuMesh, GpuVertex, MeshHandle};
use std::collections::HashMap;

pub struct MeshManager {
    meshes: Vec<GpuMesh>,
    mesh_map: HashMap<String, MeshHandle>,
}

impl MeshManager {
    pub fn new() -> Self {
        Self {
            meshes: Vec::new(),
            mesh_map: HashMap::new(),
        }
    }

    /// Upload a mesh to the GPU and return a handle
    pub fn upload_mesh(
        &mut self,
        device: &wgpu::Device,
        name: String,
        vertices: &[GpuVertex],
        indices: &[u32],
    ) -> MeshHandle {
        // Check if already uploaded
        if let Some(&handle) = self.mesh_map.get(&name) {
            return handle;
        }

        let gpu_mesh = GpuMesh::from_cpu_mesh(device, vertices, indices);
        let handle = MeshHandle(self.meshes.len());
        self.meshes.push(gpu_mesh);
        self.mesh_map.insert(name, handle);

        handle
    }

    /// Get a mesh by handle
    pub fn get_mesh(&self, handle: MeshHandle) -> Option<&GpuMesh> {
        self.meshes.get(handle.0)
    }

    /// Get a mesh handle by name
    pub fn get_handle(&self, name: &str) -> Option<MeshHandle> {
        self.mesh_map.get(name).copied()
    }

    /// Get mesh count
    pub fn mesh_count(&self) -> usize {
        self.meshes.len()
    }

    /// Clear all meshes
    pub fn clear(&mut self) {
        self.meshes.clear();
        self.mesh_map.clear();
    }
}

impl Default for MeshManager {
    fn default() -> Self {
        Self::new()
    }
}
