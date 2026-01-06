// Mesh data structure

use glam::{Vec2, Vec3};

#[derive(Debug, Clone)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub tex_coord: Vec2,
    pub color: Option<Vec3>,
}

impl Vertex {
    pub fn new(position: Vec3) -> Self {
        Self {
            position,
            normal: Vec3::Y,
            tex_coord: Vec2::ZERO,
            color: None,
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
            Vertex::new(Vec3::new(-0.5, -0.5, 0.5)).with_normal(Vec3::Z).with_color(color),
            Vertex::new(Vec3::new(0.5, -0.5, 0.5)).with_normal(Vec3::Z).with_color(color),
            Vertex::new(Vec3::new(0.5, 0.5, 0.5)).with_normal(Vec3::Z).with_color(color),
            Vertex::new(Vec3::new(-0.5, 0.5, 0.5)).with_normal(Vec3::Z).with_color(color),

            // Back face
            Vertex::new(Vec3::new(-0.5, -0.5, -0.5)).with_normal(Vec3::NEG_Z).with_color(color),
            Vertex::new(Vec3::new(0.5, -0.5, -0.5)).with_normal(Vec3::NEG_Z).with_color(color),
            Vertex::new(Vec3::new(0.5, 0.5, -0.5)).with_normal(Vec3::NEG_Z).with_color(color),
            Vertex::new(Vec3::new(-0.5, 0.5, -0.5)).with_normal(Vec3::NEG_Z).with_color(color),
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
}
