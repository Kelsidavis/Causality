// GLTF model loader

use crate::mesh::{Mesh, Vertex};
use anyhow::{Context, Result};
use glam::{Vec2, Vec3};
use std::path::Path;

pub fn load_gltf<P: AsRef<Path>>(path: P) -> Result<Vec<Mesh>> {
    let (document, buffers, _images) = gltf::import(path.as_ref())
        .with_context(|| format!("Failed to load GLTF file: {:?}", path.as_ref()))?;

    let mut meshes = Vec::new();

    for mesh in document.meshes() {
        for primitive in mesh.primitives() {
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

            // Read positions
            let positions = reader
                .read_positions()
                .context("GLTF mesh missing positions")?
                .map(|p| Vec3::from_array(p))
                .collect::<Vec<_>>();

            // Read normals (or generate default)
            let normals = reader
                .read_normals()
                .map(|normals| normals.map(|n| Vec3::from_array(n)).collect::<Vec<_>>())
                .unwrap_or_else(|| vec![Vec3::Y; positions.len()]);

            // Read texture coordinates (or generate default)
            let tex_coords = reader
                .read_tex_coords(0)
                .map(|tex_coords| {
                    tex_coords
                        .into_f32()
                        .map(|t| Vec2::from_array(t))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_else(|| vec![Vec2::ZERO; positions.len()]);

            // Read colors if available
            let colors = reader.read_colors(0).map(|colors| {
                colors
                    .into_rgb_f32()
                    .map(|c| Vec3::from_array(c))
                    .collect::<Vec<_>>()
            });

            // Build vertices
            let mut vertices = Vec::new();
            for i in 0..positions.len() {
                let mut vertex = Vertex::new(positions[i])
                    .with_normal(normals.get(i).copied().unwrap_or(Vec3::Y))
                    .with_tex_coord(tex_coords.get(i).copied().unwrap_or(Vec2::ZERO));

                if let Some(ref colors) = colors {
                    if let Some(&color) = colors.get(i) {
                        vertex = vertex.with_color(color);
                    }
                }

                vertices.push(vertex);
            }

            // Read indices
            let indices = reader
                .read_indices()
                .map(|indices| indices.into_u32().collect::<Vec<_>>())
                .unwrap_or_else(|| (0..vertices.len() as u32).collect());

            let mesh_name = mesh.name().unwrap_or("Unnamed").to_string();
            meshes.push(Mesh::new(mesh_name, vertices, indices));
        }
    }

    Ok(meshes)
}
