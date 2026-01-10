// Procedural vegetation mesh generation
//
// Generates realistic tree and bush meshes with branches and leaf clusters.

use crate::mesh::{Mesh, Vertex};
use glam::{Vec2, Vec3};
use std::f32::consts::PI;

/// Configuration for procedural tree generation
#[derive(Debug, Clone)]
pub struct TreeConfig {
    /// Height of the trunk
    pub trunk_height: f32,
    /// Radius at the base of the trunk
    pub trunk_radius: f32,
    /// Taper ratio for trunk (0.0 = same width, 1.0 = pointy)
    pub trunk_taper: f32,
    /// Number of segments around the trunk
    pub trunk_segments: u32,
    /// Color of the trunk (bark)
    pub trunk_color: Vec3,
    /// Number of branch levels
    pub branch_levels: u32,
    /// Number of branches per level
    pub branches_per_level: u32,
    /// Length of branches relative to trunk height
    pub branch_length: f32,
    /// Angle of branches from vertical (radians)
    pub branch_angle: f32,
    /// Size of leaf clusters
    pub leaf_size: f32,
    /// Number of leaf clusters per branch
    pub leaves_per_branch: u32,
    /// Color of the leaves
    pub leaf_color: Vec3,
    /// Random seed for variation
    pub seed: u32,
}

impl Default for TreeConfig {
    fn default() -> Self {
        Self {
            trunk_height: 3.0,
            trunk_radius: 0.15,
            trunk_taper: 0.4,
            trunk_segments: 8,
            trunk_color: Vec3::new(0.4, 0.25, 0.1),
            branch_levels: 3,
            branches_per_level: 5,
            branch_length: 0.4,
            branch_angle: 0.7,
            leaf_size: 0.3,
            leaves_per_branch: 3,
            leaf_color: Vec3::new(0.2, 0.5, 0.15),
            seed: 12345,
        }
    }
}

/// Configuration for procedural bush generation
#[derive(Debug, Clone)]
pub struct BushConfig {
    /// Overall radius of the bush
    pub radius: f32,
    /// Height of the bush
    pub height: f32,
    /// Number of leaf clusters
    pub cluster_count: u32,
    /// Size of each leaf cluster
    pub cluster_size: f32,
    /// Base color of the bush
    pub color: Vec3,
    /// Random seed
    pub seed: u32,
}

impl Default for BushConfig {
    fn default() -> Self {
        Self {
            radius: 0.8,
            height: 0.6,
            cluster_count: 12,
            cluster_size: 0.35,
            color: Vec3::new(0.25, 0.45, 0.2),
            seed: 54321,
        }
    }
}

// Simple pseudo-random number generator
struct SimpleRng {
    state: u32,
}

impl SimpleRng {
    fn new(seed: u32) -> Self {
        Self { state: seed.max(1) }
    }

    fn next(&mut self) -> f32 {
        self.state = self.state.wrapping_mul(1103515245).wrapping_add(12345);
        ((self.state >> 16) & 0x7FFF) as f32 / 32767.0
    }

    fn range(&mut self, min: f32, max: f32) -> f32 {
        min + self.next() * (max - min)
    }
}

/// Generate a realistic tree with branches and leaf clusters
pub fn generate_tree(config: &TreeConfig) -> Mesh {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut rng = SimpleRng::new(config.seed);

    // Generate main trunk
    generate_tapered_cylinder(
        &mut vertices,
        &mut indices,
        Vec3::ZERO,
        Vec3::Y,
        config.trunk_height,
        config.trunk_radius,
        config.trunk_radius * (1.0 - config.trunk_taper),
        config.trunk_segments,
        config.trunk_color,
    );

    // Generate branches at different levels
    for level in 0..config.branch_levels {
        let level_height = config.trunk_height * (0.4 + 0.5 * (level as f32 / config.branch_levels as f32));
        let level_radius = config.trunk_radius * (1.0 - config.trunk_taper * (level_height / config.trunk_height));

        // Branches get shorter and more numerous higher up
        let branch_count = config.branches_per_level + level;
        let branch_len = config.branch_length * config.trunk_height * (1.0 - 0.2 * level as f32);

        for b in 0..branch_count {
            let base_angle = (b as f32 / branch_count as f32) * 2.0 * PI;
            let angle_offset = rng.range(-0.3, 0.3);
            let branch_angle = base_angle + angle_offset;

            // Branch direction - outward and slightly upward
            let up_angle = config.branch_angle + rng.range(-0.2, 0.2);
            let branch_dir = Vec3::new(
                branch_angle.cos() * up_angle.sin(),
                up_angle.cos(),
                branch_angle.sin() * up_angle.sin(),
            ).normalize();

            let branch_start = Vec3::new(
                branch_angle.cos() * level_radius,
                level_height + rng.range(-0.1, 0.1),
                branch_angle.sin() * level_radius,
            );

            let branch_radius = level_radius * 0.3;

            // Generate branch
            generate_tapered_cylinder(
                &mut vertices,
                &mut indices,
                branch_start,
                branch_dir,
                branch_len,
                branch_radius,
                branch_radius * 0.3,
                6,
                config.trunk_color * 0.9,
            );

            // Generate leaf clusters along and at end of branch
            let branch_end = branch_start + branch_dir * branch_len;

            for l in 0..config.leaves_per_branch {
                let t = (l as f32 + 0.5) / config.leaves_per_branch as f32;
                let leaf_pos = branch_start + branch_dir * (branch_len * t);

                // Offset leaf cluster slightly from branch
                let offset = Vec3::new(
                    rng.range(-0.15, 0.15),
                    rng.range(0.0, 0.15),
                    rng.range(-0.15, 0.15),
                );

                let cluster_size = config.leaf_size * rng.range(0.7, 1.3);
                let color_var = config.leaf_color * rng.range(0.85, 1.15);

                generate_leaf_cluster(
                    &mut vertices,
                    &mut indices,
                    leaf_pos + offset,
                    cluster_size,
                    color_var.clamp(Vec3::ZERO, Vec3::ONE),
                    &mut rng,
                );
            }

            // Extra cluster at branch end
            generate_leaf_cluster(
                &mut vertices,
                &mut indices,
                branch_end + Vec3::new(rng.range(-0.1, 0.1), 0.05, rng.range(-0.1, 0.1)),
                config.leaf_size * 1.2,
                config.leaf_color * rng.range(0.9, 1.1),
                &mut rng,
            );
        }
    }

    // Add some leaf clusters at the top of the trunk
    let top_pos = Vec3::new(0.0, config.trunk_height, 0.0);
    for i in 0..5 {
        let angle = (i as f32 / 5.0) * 2.0 * PI + rng.range(0.0, 0.5);
        let offset = Vec3::new(
            angle.cos() * rng.range(0.1, 0.3),
            rng.range(0.0, 0.3),
            angle.sin() * rng.range(0.1, 0.3),
        );
        generate_leaf_cluster(
            &mut vertices,
            &mut indices,
            top_pos + offset,
            config.leaf_size * rng.range(0.8, 1.2),
            config.leaf_color * rng.range(0.9, 1.1),
            &mut rng,
        );
    }

    let mut mesh = Mesh::new("ProceduralTree".to_string(), vertices, indices);
    mesh.calculate_tangents();
    mesh
}

/// Generate a bush made of overlapping leaf clusters
pub fn generate_bush(config: &BushConfig) -> Mesh {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut rng = SimpleRng::new(config.seed);

    // Generate multiple leaf clusters to form the bush
    for i in 0..config.cluster_count {
        // Distribute clusters in a hemisphere shape
        let u = rng.next();
        let v = rng.next();

        let theta = u * 2.0 * PI;
        let phi = v * PI * 0.5; // Hemisphere

        let r = config.radius * (0.5 + 0.5 * rng.next()); // Vary distance from center

        let x = r * phi.sin() * theta.cos();
        let z = r * phi.sin() * theta.sin();
        let y = config.height * phi.cos() * (0.3 + 0.7 * rng.next());

        let pos = Vec3::new(x, y.max(0.05), z);
        let size = config.cluster_size * rng.range(0.7, 1.3);
        let color = config.color * rng.range(0.85, 1.15);

        generate_leaf_cluster(
            &mut vertices,
            &mut indices,
            pos,
            size,
            color.clamp(Vec3::ZERO, Vec3::ONE),
            &mut rng,
        );
    }

    // Add a few clusters at ground level for fullness
    for i in 0..4 {
        let angle = (i as f32 / 4.0) * 2.0 * PI + rng.range(0.0, 0.5);
        let r = config.radius * rng.range(0.6, 0.9);
        let pos = Vec3::new(
            angle.cos() * r,
            config.height * 0.15,
            angle.sin() * r,
        );
        generate_leaf_cluster(
            &mut vertices,
            &mut indices,
            pos,
            config.cluster_size * 0.8,
            config.color * rng.range(0.8, 1.0),
            &mut rng,
        );
    }

    let mut mesh = Mesh::new("ProceduralBush".to_string(), vertices, indices);
    mesh.calculate_tangents();
    mesh
}

/// Generate a pine tree with tiered branches
pub fn generate_pine_tree() -> Mesh {
    let config = TreeConfig {
        trunk_height: 4.0,
        trunk_radius: 0.12,
        trunk_taper: 0.5,
        trunk_segments: 8,
        trunk_color: Vec3::new(0.35, 0.2, 0.1),
        branch_levels: 5,
        branches_per_level: 6,
        branch_length: 0.35,
        branch_angle: 0.9, // More horizontal for pine
        leaf_size: 0.25,
        leaves_per_branch: 2,
        leaf_color: Vec3::new(0.1, 0.35, 0.15),
        seed: 11111,
    };
    generate_tree(&config)
}

/// Generate an oak-style deciduous tree
pub fn generate_oak_tree() -> Mesh {
    let config = TreeConfig {
        trunk_height: 3.5,
        trunk_radius: 0.2,
        trunk_taper: 0.35,
        trunk_segments: 10,
        trunk_color: Vec3::new(0.45, 0.3, 0.15),
        branch_levels: 4,
        branches_per_level: 4,
        branch_length: 0.5,
        branch_angle: 0.6, // More upward for oak
        leaf_size: 0.4,
        leaves_per_branch: 4,
        leaf_color: Vec3::new(0.25, 0.5, 0.2),
        seed: 22222,
    };
    generate_tree(&config)
}

/// Generate a small bush
pub fn generate_small_bush() -> Mesh {
    let config = BushConfig {
        radius: 0.5,
        height: 0.4,
        cluster_count: 8,
        cluster_size: 0.25,
        color: Vec3::new(0.3, 0.5, 0.25),
        seed: 33333,
    };
    generate_bush(&config)
}

/// Generate a larger shrub
pub fn generate_shrub() -> Mesh {
    let config = BushConfig {
        radius: 1.0,
        height: 0.8,
        cluster_count: 16,
        cluster_size: 0.35,
        color: Vec3::new(0.2, 0.4, 0.15),
        seed: 44444,
    };
    generate_bush(&config)
}

// ============ Internal generation functions ============

fn generate_tapered_cylinder(
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u32>,
    start: Vec3,
    direction: Vec3,
    length: f32,
    radius_start: f32,
    radius_end: f32,
    segments: u32,
    color: Vec3,
) {
    let base_index = vertices.len() as u32;

    // Create orthonormal basis for the cylinder
    let up = if direction.y.abs() > 0.99 {
        Vec3::X
    } else {
        Vec3::Y
    };
    let right = direction.cross(up).normalize();
    let forward = right.cross(direction).normalize();

    let end = start + direction * length;

    // Generate vertices for cylinder
    for i in 0..=segments {
        let angle = (i as f32 / segments as f32) * 2.0 * PI;
        let cos_a = angle.cos();
        let sin_a = angle.sin();

        let normal = (right * cos_a + forward * sin_a).normalize();
        let u = i as f32 / segments as f32;

        // Bottom vertex
        let bottom_offset = (right * cos_a + forward * sin_a) * radius_start;
        vertices.push(
            Vertex::new(start + bottom_offset)
                .with_normal(normal)
                .with_tex_coord(Vec2::new(u, 0.0))
                .with_color(color * 0.85),
        );

        // Top vertex
        let top_offset = (right * cos_a + forward * sin_a) * radius_end;
        vertices.push(
            Vertex::new(end + top_offset)
                .with_normal(normal)
                .with_tex_coord(Vec2::new(u, 1.0))
                .with_color(color),
        );
    }

    // Generate indices
    for i in 0..segments {
        let bl = base_index + i * 2;
        let br = base_index + (i + 1) * 2;
        let tl = bl + 1;
        let tr = br + 1;

        indices.extend_from_slice(&[bl, br, tr, tr, tl, bl]);
    }
}

fn generate_leaf_cluster(
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u32>,
    center: Vec3,
    size: f32,
    color: Vec3,
    rng: &mut SimpleRng,
) {
    let base_index = vertices.len() as u32;

    // Generate a bumpy spheroid for leaf cluster
    let segments = 8u32;
    let rings = 5u32;

    for ring in 0..=rings {
        let v = ring as f32 / rings as f32;
        let phi = v * PI;
        let y = phi.cos() * size * 0.7; // Slightly flattened
        let ring_radius = phi.sin() * size;

        for seg in 0..=segments {
            let u = seg as f32 / segments as f32;
            let theta = u * 2.0 * PI;

            // Add some bumpiness
            let bump = 1.0 + (rng.next() - 0.5) * 0.3;
            let x = theta.cos() * ring_radius * bump;
            let z = theta.sin() * ring_radius * bump;

            let pos = center + Vec3::new(x, y, z);
            let normal = Vec3::new(x, y * 1.4, z).normalize(); // Adjust normal for flattening

            // Color variation
            let color_var = color * (0.9 + rng.next() * 0.2);

            vertices.push(
                Vertex::new(pos)
                    .with_normal(normal)
                    .with_tex_coord(Vec2::new(u, v))
                    .with_color(color_var.clamp(Vec3::ZERO, Vec3::ONE)),
            );
        }
    }

    // Generate indices
    let verts_per_ring = segments + 1;
    for ring in 0..rings {
        for seg in 0..segments {
            let current = base_index + ring * verts_per_ring + seg;
            let next = current + 1;
            let below = current + verts_per_ring;
            let below_next = below + 1;

            indices.extend_from_slice(&[current, below, below_next, below_next, next, current]);
        }
    }
}

/// Vegetation type for the foliage system
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VegetationType {
    PineTree,
    OakTree,
    Bush,
    Shrub,
}

impl VegetationType {
    pub fn all() -> &'static [VegetationType] {
        &[
            VegetationType::PineTree,
            VegetationType::OakTree,
            VegetationType::Bush,
            VegetationType::Shrub,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            VegetationType::PineTree => "Pine Tree",
            VegetationType::OakTree => "Oak Tree",
            VegetationType::Bush => "Bush",
            VegetationType::Shrub => "Shrub",
        }
    }

    pub fn mesh_name(&self) -> &'static str {
        match self {
            VegetationType::PineTree => "vegetation_pine",
            VegetationType::OakTree => "vegetation_oak",
            VegetationType::Bush => "vegetation_bush",
            VegetationType::Shrub => "vegetation_shrub",
        }
    }

    pub fn generate_mesh(&self) -> Mesh {
        match self {
            VegetationType::PineTree => generate_pine_tree(),
            VegetationType::OakTree => generate_oak_tree(),
            VegetationType::Bush => generate_small_bush(),
            VegetationType::Shrub => generate_shrub(),
        }
    }
}
