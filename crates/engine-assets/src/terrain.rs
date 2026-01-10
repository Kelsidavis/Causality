// Terrain generation with height maps

use crate::mesh::{Mesh, Vertex};
use glam::{Vec2, Vec3};
use noise::{NoiseFn, Perlin, Seedable};

#[derive(Clone)]
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

    /// Generate terrain with deliberate lake basins for water containment
    /// Creates a terrain with raised edges and depressions in the interior
    pub fn generate_with_basins(config: &TerrainConfig, num_basins: usize) -> Self {
        let perlin = Perlin::new(config.seed);
        let mut heights = Vec::with_capacity(config.width * config.depth);

        // Define basin locations (in normalized 0-1 coords)
        let basins: Vec<(f64, f64, f64, f64)> = (0..num_basins)
            .map(|i| {
                let seed_offset = i as f64 * 0.7;
                let bx = 0.2 + 0.6 * ((perlin.get([seed_offset, 0.0]) + 1.0) * 0.5);
                let bz = 0.2 + 0.6 * ((perlin.get([0.0, seed_offset]) + 1.0) * 0.5);
                let radius = 0.1 + 0.15 * ((perlin.get([seed_offset, seed_offset]) + 1.0) * 0.5);
                let depth = 0.3 + 0.4 * ((perlin.get([seed_offset * 2.0, 0.0]) + 1.0) * 0.5);
                (bx, bz, radius, depth)
            })
            .collect();

        for z in 0..config.depth {
            for x in 0..config.width {
                let nx = x as f64 / config.width as f64;
                let nz = z as f64 / config.depth as f64;

                // Base terrain from noise
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

                height = (height + 1.0) * 0.5;

                // Raise edges to contain water (bowl shape)
                let edge_x = (nx - 0.5).abs() * 2.0; // 0 at center, 1 at edge
                let edge_z = (nz - 0.5).abs() * 2.0;
                let edge_dist = (edge_x.max(edge_z)).powf(2.0);
                height += edge_dist * 0.3; // Raise edges

                // Carve basins (depressions)
                for &(bx, bz, radius, basin_depth) in &basins {
                    let dx = nx - bx;
                    let dz = nz - bz;
                    let dist = (dx * dx + dz * dz).sqrt();
                    if dist < radius {
                        // Smooth basin profile (parabolic)
                        let t = dist / radius;
                        let depression = (1.0 - t * t) * basin_depth;
                        height -= depression;
                    }
                }

                heights.push(height as f32 * config.height_scale);
            }
        }

        Self {
            width: config.width,
            depth: config.depth,
            heights,
        }
    }

    /// Generate terrain with a castle moat - flat ground with ring-shaped trench
    pub fn generate_with_moat(config: &TerrainConfig, inner_radius: f64, outer_radius: f64, moat_depth: f64) -> Self {
        let perlin = Perlin::new(config.seed);
        let mut heights = Vec::with_capacity(config.width * config.depth);

        for z in 0..config.depth {
            for x in 0..config.width {
                let nx = x as f64 / config.width as f64;
                let nz = z as f64 / config.depth as f64;

                // Distance from center (normalized 0-1 at edges)
                let dx = (nx - 0.5) * 2.0;
                let dz = (nz - 0.5) * 2.0;
                let dist_from_center = (dx * dx + dz * dz).sqrt();

                // Base terrain - flat with subtle noise
                let mut height = 0.5; // Start at middle height

                // Add very subtle noise for visual interest (outside moat only)
                if dist_from_center > outer_radius {
                    let noise_val = perlin.get([nx * config.frequency, nz * config.frequency]);
                    height += noise_val * 0.05;
                }

                // Moat ring - trench between inner and outer radius
                if dist_from_center >= inner_radius && dist_from_center < outer_radius {
                    let moat_width = outer_radius - inner_radius;
                    let t = (dist_from_center - inner_radius) / moat_width;
                    // Smooth moat profile (deepest in middle)
                    let moat_profile = (t * std::f64::consts::PI).sin();
                    height -= moat_depth * moat_profile;
                }

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

    /// Set height at grid position
    pub fn set_height(&mut self, x: usize, z: usize, height: f32) {
        if x < self.width && z < self.depth {
            self.heights[z * self.width + x] = height;
        }
    }

    /// Convert world coordinates to grid coordinates
    pub fn world_to_grid(&self, world_x: f32, world_z: f32, scale: f32) -> (f32, f32) {
        let grid_x = (world_x / scale) * self.width as f32 + (self.width as f32 * 0.5);
        let grid_z = (world_z / scale) * self.depth as f32 + (self.depth as f32 * 0.5);
        (grid_x, grid_z)
    }

    /// Apply a brush operation at world position
    /// brush_mode: 0=raise, 1=lower, 2=smooth, 3=flatten
    /// Returns true if any heights were modified
    pub fn apply_brush(
        &mut self,
        world_x: f32,
        world_z: f32,
        scale: f32,
        radius: f32,
        strength: f32,
        brush_mode: u8,
    ) -> bool {
        // Convert world position to grid position
        let (center_x, center_z) = self.world_to_grid(world_x, world_z, scale);

        // Convert radius from world units to grid units
        let grid_radius = (radius / scale) * self.width as f32;

        // Calculate affected grid cells
        let min_x = ((center_x - grid_radius).floor() as i32).max(0) as usize;
        let max_x = ((center_x + grid_radius).ceil() as i32).min(self.width as i32 - 1) as usize;
        let min_z = ((center_z - grid_radius).floor() as i32).max(0) as usize;
        let max_z = ((center_z + grid_radius).ceil() as i32).min(self.depth as i32 - 1) as usize;

        // For flatten mode, get the center height first
        let flatten_height = if brush_mode == 3 {
            let cx = center_x.round() as usize;
            let cz = center_z.round() as usize;
            if cx < self.width && cz < self.depth {
                self.get_height(cx, cz)
            } else {
                0.0
            }
        } else {
            0.0
        };

        // For smooth mode, we need to pre-calculate smoothed values
        let smooth_values = if brush_mode == 2 {
            let mut values = Vec::new();
            for z in min_z..=max_z {
                for x in min_x..=max_x {
                    let avg = self.get_average_height(x, z);
                    values.push(avg);
                }
            }
            values
        } else {
            Vec::new()
        };

        let mut modified = false;
        let mut smooth_idx = 0;

        for z in min_z..=max_z {
            for x in min_x..=max_x {
                let dx = x as f32 - center_x;
                let dz = z as f32 - center_z;
                let dist = (dx * dx + dz * dz).sqrt();

                if dist <= grid_radius {
                    // Smooth falloff (1 at center, 0 at edge)
                    let falloff = 1.0 - (dist / grid_radius);
                    let falloff = falloff * falloff; // Quadratic falloff for smoother edges

                    let current_height = self.get_height(x, z);
                    let delta = strength * falloff * 0.1; // Scale down for finer control

                    let new_height = match brush_mode {
                        0 => current_height + delta, // Raise
                        1 => current_height - delta, // Lower
                        2 => {
                            // Smooth - blend toward average
                            let avg = smooth_values[smooth_idx];
                            current_height + (avg - current_height) * falloff * strength * 0.5
                        }
                        3 => {
                            // Flatten - blend toward center height
                            current_height + (flatten_height - current_height) * falloff * strength * 0.5
                        }
                        _ => current_height,
                    };

                    self.set_height(x, z, new_height);
                    modified = true;
                }

                if brush_mode == 2 {
                    smooth_idx += 1;
                }
            }
        }

        modified
    }

    /// Get average height of a cell and its neighbors (for smoothing)
    fn get_average_height(&self, x: usize, z: usize) -> f32 {
        let mut sum = 0.0;
        let mut count = 0;

        for dz in -1i32..=1 {
            for dx in -1i32..=1 {
                let nx = x as i32 + dx;
                let nz = z as i32 + dz;
                if nx >= 0 && nx < self.width as i32 && nz >= 0 && nz < self.depth as i32 {
                    sum += self.get_height(nx as usize, nz as usize);
                    count += 1;
                }
            }
        }

        if count > 0 { sum / count as f32 } else { 0.0 }
    }
}

pub struct Terrain;

impl Terrain {
    /// Generate terrain mesh from an existing height map
    pub fn generate_mesh_from_heightmap(height_map: &HeightMap, config: &TerrainConfig) -> Mesh {
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
                let normal = Self::calculate_normal(height_map, x, z, cell_size);

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

    /// Generate terrain mesh from height map (generates a new heightmap)
    pub fn generate_mesh(config: &TerrainConfig) -> Mesh {
        let height_map = HeightMap::generate(config);
        Self::generate_mesh_from_heightmap(&height_map, config)
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
