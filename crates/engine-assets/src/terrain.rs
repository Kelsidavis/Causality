// Terrain generation with height maps

use crate::mesh::{Mesh, Vertex};
use glam::{Vec2, Vec3};
use noise::{NoiseFn, Perlin, Seedable};

pub struct TerrainConfig {
    pub width: usize,      // Number of vertices in X direction
    pub depth: usize,      // Number of vertices in Z direction
    pub scale: f32,        // World-space size
    pub height_scale: f32, // Height multiplier
    pub seed: u32,         // Random seed for noise
    pub octaves: usize,    // Number of noise octaves
    pub frequency: f64,    // Base frequency of noise
    pub lacunarity: f64,   // Frequency multiplier per octave
    pub persistence: f64,  // Amplitude multiplier per octave
}

impl Default for TerrainConfig {
    fn default() -> Self {
        Self {
            width: 64,
            depth: 64,
            scale: 50.0,
            height_scale: 5.0,
            seed: 0,
            octaves: 4,
            frequency: 1.0,
            lacunarity: 2.0,
            persistence: 0.5,
        }
    }
}

pub struct HeightMap {
    pub width: usize,
    pub depth: usize,
    pub heights: Vec<f32>,
}

impl HeightMap {
    /// Generate height map using Perlin noise
    pub fn generate(config: &TerrainConfig) -> Self {
        let perlin = Perlin::new(config.seed);
        let mut heights = Vec::with_capacity(config.width * config.depth);

        for z in 0..config.depth {
            for x in 0..config.width {
                // Normalize coordinates to [0, 1]
                let nx = x as f64 / config.width as f64;
                let nz = z as f64 / config.depth as f64;

                // Multi-octave noise (fractal Brownian motion)
                let mut height = 0.0;
                let mut amplitude = 1.0;
                let mut frequency = config.frequency;

                for _ in 0..config.octaves {
                    let sample_x = nx * frequency;
                    let sample_z = nz * frequency;
                    let noise_value = perlin.get([sample_x, sample_z]);
                    height += noise_value * amplitude;

                    amplitude *= config.persistence;
                    frequency *= config.lacunarity;
                }

                // Normalize from [-1, 1] to [0, 1] and scale
                height = (height + 1.0) * 0.5;
                heights.push(height as f32 * config.height_scale);
            }
        }

        Self {
            width: config.width,
            depth: config.depth,
            heights,
        }
    }

    /// Get height at grid position
    pub fn get_height(&self, x: usize, z: usize) -> f32 {
        if x >= self.width || z >= self.depth {
            return 0.0;
        }
        self.heights[z * self.width + x]
    }

    /// Get interpolated height at world position
    pub fn sample_height(&self, world_x: f32, world_z: f32, scale: f32) -> f32 {
        // Convert world coordinates to grid coordinates
        let grid_x = (world_x / scale) * self.width as f32 + (self.width as f32 * 0.5);
        let grid_z = (world_z / scale) * self.depth as f32 + (self.depth as f32 * 0.5);

        // Get integer grid positions
        let x0 = grid_x.floor() as usize;
        let z0 = grid_z.floor() as usize;
        let x1 = (x0 + 1).min(self.width - 1);
        let z1 = (z0 + 1).min(self.depth - 1);

        // Get fractional parts
        let fx = grid_x - x0 as f32;
        let fz = grid_z - z0 as f32;

        // Bilinear interpolation
        let h00 = self.get_height(x0, z0);
        let h10 = self.get_height(x1, z0);
        let h01 = self.get_height(x0, z1);
        let h11 = self.get_height(x1, z1);

        let h0 = h00 * (1.0 - fx) + h10 * fx;
        let h1 = h01 * (1.0 - fx) + h11 * fx;

        h0 * (1.0 - fz) + h1 * fz
    }
}

pub struct Terrain;

impl Terrain {
    /// Generate terrain mesh from height map
    pub fn generate_mesh(config: &TerrainConfig) -> Mesh {
        let height_map = HeightMap::generate(config);
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        let cell_size = config.scale / config.width as f32;
        let offset_x = -config.scale * 0.5;
        let offset_z = -config.scale * 0.5;

        // Generate vertices
        for z in 0..config.depth {
            for x in 0..config.width {
                let world_x = offset_x + x as f32 * cell_size;
                let world_z = offset_z + z as f32 * cell_size;
                let height = height_map.get_height(x, z);

                // Calculate normal using neighboring heights
                let normal = Self::calculate_normal(&height_map, x, z, cell_size);

                // UV coordinates
                let u = x as f32 / (config.width - 1) as f32;
                let v = z as f32 / (config.depth - 1) as f32;

                vertices.push(
                    Vertex::new(Vec3::new(world_x, height, world_z))
                        .with_normal(normal)
                        .with_tex_coord(Vec2::new(u, v))
                        .with_color(Vec3::ONE),
                );
            }
        }

        // Generate indices (two triangles per quad)
        for z in 0..(config.depth - 1) {
            for x in 0..(config.width - 1) {
                let top_left = (z * config.width + x) as u32;
                let top_right = top_left + 1;
                let bottom_left = ((z + 1) * config.width + x) as u32;
                let bottom_right = bottom_left + 1;

                // First triangle (top-left, bottom-left, top-right)
                indices.push(top_left);
                indices.push(bottom_left);
                indices.push(top_right);

                // Second triangle (top-right, bottom-left, bottom-right)
                indices.push(top_right);
                indices.push(bottom_left);
                indices.push(bottom_right);
            }
        }

        Mesh::new("Terrain".to_string(), vertices, indices)
    }

    /// Calculate vertex normal using finite differences
    fn calculate_normal(height_map: &HeightMap, x: usize, z: usize, cell_size: f32) -> Vec3 {
        let h_center = height_map.get_height(x, z);

        // Get neighboring heights
        let h_left = if x > 0 {
            height_map.get_height(x - 1, z)
        } else {
            h_center
        };
        let h_right = if x < height_map.width - 1 {
            height_map.get_height(x + 1, z)
        } else {
            h_center
        };
        let h_up = if z > 0 {
            height_map.get_height(x, z - 1)
        } else {
            h_center
        };
        let h_down = if z < height_map.depth - 1 {
            height_map.get_height(x, z + 1)
        } else {
            h_center
        };

        // Calculate tangent vectors
        let dx = Vec3::new(cell_size * 2.0, h_right - h_left, 0.0);
        let dz = Vec3::new(0.0, h_down - h_up, cell_size * 2.0);

        // Normal is cross product of tangents
        dx.cross(dz).normalize()
    }
}
