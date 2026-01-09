// Mesh data structure

use glam::{Vec2, Vec3, Vec4};

#[derive(Debug, Clone)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub tex_coord: Vec2,
    pub color: Option<Vec3>,
    pub tangent: Option<Vec4>,      // w component is handedness (+1 or -1)
    pub bitangent: Option<Vec3>,
}

impl Vertex {
    pub fn new(position: Vec3) -> Self {
        Self {
            position,
            normal: Vec3::Y,
            tex_coord: Vec2::ZERO,
            color: None,
            tangent: None,
            bitangent: None,
        }
    }

    pub fn with_normal(mut self, normal: Vec3) -> Self {
        self.normal = normal;
        self
    }

    pub fn with_tex_coord(mut self, tex_coord: Vec2) -> Self {
        self.tex_coord = tex_coord;
        self
    }

    pub fn with_color(mut self, color: Vec3) -> Self {
        self.color = Some(color);
        self
    }

    pub fn with_tangent(mut self, tangent: Vec4) -> Self {
        self.tangent = Some(tangent);
        self
    }

    pub fn with_bitangent(mut self, bitangent: Vec3) -> Self {
        self.bitangent = Some(bitangent);
        self
    }
}

#[derive(Debug, Clone)]
pub struct Mesh {
    pub name: String,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl Mesh {
    pub fn new(name: String, vertices: Vec<Vertex>, indices: Vec<u32>) -> Self {
        Self {
            name,
            vertices,
            indices,
        }
    }

    /// Create a simple cube mesh
    pub fn cube() -> Self {
        let vertices = vec![
            // Front face
            Vertex::new(Vec3::new(-0.5, -0.5, 0.5)).with_normal(Vec3::Z).with_color(Vec3::new(1.0, 0.0, 0.0)),
            Vertex::new(Vec3::new(0.5, -0.5, 0.5)).with_normal(Vec3::Z).with_color(Vec3::new(0.0, 1.0, 0.0)),
            Vertex::new(Vec3::new(0.5, 0.5, 0.5)).with_normal(Vec3::Z).with_color(Vec3::new(0.0, 0.0, 1.0)),
            Vertex::new(Vec3::new(-0.5, 0.5, 0.5)).with_normal(Vec3::Z).with_color(Vec3::new(1.0, 1.0, 0.0)),

            // Back face
            Vertex::new(Vec3::new(-0.5, -0.5, -0.5)).with_normal(Vec3::NEG_Z).with_color(Vec3::new(1.0, 0.0, 1.0)),
            Vertex::new(Vec3::new(0.5, -0.5, -0.5)).with_normal(Vec3::NEG_Z).with_color(Vec3::new(0.0, 1.0, 1.0)),
            Vertex::new(Vec3::new(0.5, 0.5, -0.5)).with_normal(Vec3::NEG_Z).with_color(Vec3::new(1.0, 1.0, 1.0)),
            Vertex::new(Vec3::new(-0.5, 0.5, -0.5)).with_normal(Vec3::NEG_Z).with_color(Vec3::new(0.5, 0.5, 0.5)),
        ];

        let indices = vec![
            0, 1, 2, 2, 3, 0, // front
            1, 5, 6, 6, 2, 1, // right
            5, 4, 7, 7, 6, 5, // back
            4, 0, 3, 3, 7, 4, // left
            3, 2, 6, 6, 7, 3, // top
            4, 5, 1, 1, 0, 4, // bottom
        ];

        Self::new("Cube".to_string(), vertices, indices)
    }

    /// Create a cube mesh with a specific color
    pub fn cube_with_color(color: Vec3) -> Self {
        let vertices = vec![
            // Front face
            Vertex::new(Vec3::new(-0.5, -0.5, 0.5)).with_normal(Vec3::Z).with_tex_coord(Vec2::new(0.0, 0.0)).with_color(color),
            Vertex::new(Vec3::new(0.5, -0.5, 0.5)).with_normal(Vec3::Z).with_tex_coord(Vec2::new(1.0, 0.0)).with_color(color),
            Vertex::new(Vec3::new(0.5, 0.5, 0.5)).with_normal(Vec3::Z).with_tex_coord(Vec2::new(1.0, 1.0)).with_color(color),
            Vertex::new(Vec3::new(-0.5, 0.5, 0.5)).with_normal(Vec3::Z).with_tex_coord(Vec2::new(0.0, 1.0)).with_color(color),

            // Back face
            Vertex::new(Vec3::new(-0.5, -0.5, -0.5)).with_normal(Vec3::NEG_Z).with_tex_coord(Vec2::new(1.0, 0.0)).with_color(color),
            Vertex::new(Vec3::new(0.5, -0.5, -0.5)).with_normal(Vec3::NEG_Z).with_tex_coord(Vec2::new(0.0, 0.0)).with_color(color),
            Vertex::new(Vec3::new(0.5, 0.5, -0.5)).with_normal(Vec3::NEG_Z).with_tex_coord(Vec2::new(0.0, 1.0)).with_color(color),
            Vertex::new(Vec3::new(-0.5, 0.5, -0.5)).with_normal(Vec3::NEG_Z).with_tex_coord(Vec2::new(1.0, 1.0)).with_color(color),

            // Right face
            Vertex::new(Vec3::new(0.5, -0.5, 0.5)).with_normal(Vec3::X).with_tex_coord(Vec2::new(0.0, 0.0)).with_color(color),
            Vertex::new(Vec3::new(0.5, -0.5, -0.5)).with_normal(Vec3::X).with_tex_coord(Vec2::new(1.0, 0.0)).with_color(color),
            Vertex::new(Vec3::new(0.5, 0.5, -0.5)).with_normal(Vec3::X).with_tex_coord(Vec2::new(1.0, 1.0)).with_color(color),
            Vertex::new(Vec3::new(0.5, 0.5, 0.5)).with_normal(Vec3::X).with_tex_coord(Vec2::new(0.0, 1.0)).with_color(color),

            // Left face
            Vertex::new(Vec3::new(-0.5, -0.5, -0.5)).with_normal(Vec3::NEG_X).with_tex_coord(Vec2::new(0.0, 0.0)).with_color(color),
            Vertex::new(Vec3::new(-0.5, -0.5, 0.5)).with_normal(Vec3::NEG_X).with_tex_coord(Vec2::new(1.0, 0.0)).with_color(color),
            Vertex::new(Vec3::new(-0.5, 0.5, 0.5)).with_normal(Vec3::NEG_X).with_tex_coord(Vec2::new(1.0, 1.0)).with_color(color),
            Vertex::new(Vec3::new(-0.5, 0.5, -0.5)).with_normal(Vec3::NEG_X).with_tex_coord(Vec2::new(0.0, 1.0)).with_color(color),

            // Top face
            Vertex::new(Vec3::new(-0.5, 0.5, 0.5)).with_normal(Vec3::Y).with_tex_coord(Vec2::new(0.0, 0.0)).with_color(color),
            Vertex::new(Vec3::new(0.5, 0.5, 0.5)).with_normal(Vec3::Y).with_tex_coord(Vec2::new(1.0, 0.0)).with_color(color),
            Vertex::new(Vec3::new(0.5, 0.5, -0.5)).with_normal(Vec3::Y).with_tex_coord(Vec2::new(1.0, 1.0)).with_color(color),
            Vertex::new(Vec3::new(-0.5, 0.5, -0.5)).with_normal(Vec3::Y).with_tex_coord(Vec2::new(0.0, 1.0)).with_color(color),

            // Bottom face
            Vertex::new(Vec3::new(-0.5, -0.5, -0.5)).with_normal(Vec3::NEG_Y).with_tex_coord(Vec2::new(0.0, 1.0)).with_color(color),
            Vertex::new(Vec3::new(0.5, -0.5, -0.5)).with_normal(Vec3::NEG_Y).with_tex_coord(Vec2::new(1.0, 1.0)).with_color(color),
            Vertex::new(Vec3::new(0.5, -0.5, 0.5)).with_normal(Vec3::NEG_Y).with_tex_coord(Vec2::new(1.0, 0.0)).with_color(color),
            Vertex::new(Vec3::new(-0.5, -0.5, 0.5)).with_normal(Vec3::NEG_Y).with_tex_coord(Vec2::new(0.0, 0.0)).with_color(color),
        ];

        let indices = vec![
            0, 1, 2, 2, 3, 0,       // front
            8, 9, 10, 10, 11, 8,    // right
            12, 13, 14, 14, 15, 12, // left
            16, 17, 18, 18, 19, 16, // top
            20, 21, 22, 22, 23, 20, // bottom
            4, 5, 6, 6, 7, 4,       // back
        ];

        Self::new("Cube".to_string(), vertices, indices)
    }

    /// Create a plane mesh
    pub fn plane(size: f32) -> Self {
        let half = size / 2.0;
        let vertices = vec![
            Vertex::new(Vec3::new(-half, 0.0, -half)).with_normal(Vec3::Y).with_tex_coord(Vec2::new(0.0, 0.0)),
            Vertex::new(Vec3::new(half, 0.0, -half)).with_normal(Vec3::Y).with_tex_coord(Vec2::new(1.0, 0.0)),
            Vertex::new(Vec3::new(half, 0.0, half)).with_normal(Vec3::Y).with_tex_coord(Vec2::new(1.0, 1.0)),
            Vertex::new(Vec3::new(-half, 0.0, half)).with_normal(Vec3::Y).with_tex_coord(Vec2::new(0.0, 1.0)),
        ];

        let indices = vec![0, 1, 2, 2, 3, 0];

        Self::new("Plane".to_string(), vertices, indices)
    }

    /// Calculate tangents and bitangents using mikktspace algorithm
    /// This is the industry-standard approach used by most 3D tools
    pub fn calculate_tangents(&mut self) {
        // Skip if no UVs (tangent space requires texture coordinates)
        if self.vertices.is_empty() || self.indices.is_empty() {
            log::warn!("Cannot calculate tangents: mesh has no vertices or indices");
            return;
        }

        // Check if all vertices have valid UVs
        let has_uvs = self.vertices.iter().all(|v| v.tex_coord != Vec2::ZERO);
        if !has_uvs {
            log::warn!("Cannot calculate tangents: mesh has no valid UVs");
            return;
        }

        // Generate tangents using mikktspace
        let tangents = {
            let mut context = MikkTSpaceContext {
                mesh: self,
                tangents: vec![Vec4::ZERO; self.vertices.len()],
            };

            let result = mikktspace::generate_tangents(&mut context);

            if !result {
                log::warn!("mikktspace failed to generate tangents for mesh: {}", self.name);
                return;
            }

            context.tangents
        }; // context is dropped here, releasing the immutable borrow

        // Apply tangents and calculate bitangents
        for (i, vertex) in self.vertices.iter_mut().enumerate() {
            let tangent = tangents[i];
            vertex.tangent = Some(tangent);

            // Calculate bitangent: B = (N × T) * handedness
            let n = vertex.normal;
            let t = tangent.truncate(); // Vec4 -> Vec3 (ignore w)
            let handedness = tangent.w;
            vertex.bitangent = Some(n.cross(t) * handedness);
        }

        log::info!("Generated tangents for mesh: {}", self.name);
    }
}

// mikktspace integration
struct MikkTSpaceContext<'a> {
    mesh: &'a Mesh,
    tangents: Vec<Vec4>,
}

impl<'a> mikktspace::Geometry for MikkTSpaceContext<'a> {
    fn num_faces(&self) -> usize {
        self.mesh.indices.len() / 3
    }

    fn num_vertices_of_face(&self, _face: usize) -> usize {
        3 // Always triangles
    }

    fn position(&self, face: usize, vert: usize) -> [f32; 3] {
        let index = self.mesh.indices[face * 3 + vert] as usize;
        let pos = self.mesh.vertices[index].position;
        [pos.x, pos.y, pos.z]
    }

    fn normal(&self, face: usize, vert: usize) -> [f32; 3] {
        let index = self.mesh.indices[face * 3 + vert] as usize;
        let normal = self.mesh.vertices[index].normal;
        [normal.x, normal.y, normal.z]
    }

    fn tex_coord(&self, face: usize, vert: usize) -> [f32; 2] {
        let index = self.mesh.indices[face * 3 + vert] as usize;
        let uv = self.mesh.vertices[index].tex_coord;
        [uv.x, uv.y]
    }

    fn set_tangent_encoded(&mut self, tangent: [f32; 4], face: usize, vert: usize) {
        let index = self.mesh.indices[face * 3 + vert] as usize;
        self.tangents[index] = Vec4::from_array(tangent);
    }
}
