// Terrain-aware water fill using priority-flood algorithm
// Computes water levels that fill terrain depressions

use crate::mesh::{Mesh, Vertex};
use crate::terrain::{HeightMap, TerrainConfig};
use glam::{Vec2, Vec3};
use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};
use std::cmp::Ordering;

/// Cell in priority queue for flood-fill
#[derive(Clone, Copy)]
struct FloodCell {
    x: usize,
    z: usize,
    height: f32,
}

impl Ord for FloodCell {
    fn cmp(&self, other: &Self) -> Ordering {
        // Min-heap: lower heights have higher priority
        other.height.partial_cmp(&self.height).unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for FloodCell {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for FloodCell {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.z == other.z
    }
}

impl Eq for FloodCell {}

/// A computed water body from flood-fill
#[derive(Debug, Clone)]
pub struct ComputedWaterBody {
    pub id: u32,
    pub surface_level: f32,
    pub cells: Vec<(usize, usize)>,
    pub bounds_min: Vec3,
    pub bounds_max: Vec3,
    pub average_depth: f32,
    pub connected_to: Vec<u32>,
    pub flow_direction: Option<[f32; 2]>,
    pub flow_speed: f32,
}

/// Result of flood-fill water computation
pub struct WaterFillResult {
    /// Water level at each grid cell (terrain height if no water)
    pub water_levels: Vec<f32>,
    pub width: usize,
    pub depth: usize,
    /// Identified water bodies
    pub water_bodies: Vec<ComputedWaterBody>,
}

/// 8-connected neighbors
const NEIGHBORS: [(i32, i32); 8] = [
    (-1, -1), (0, -1), (1, -1),
    (-1,  0),          (1,  0),
    (-1,  1), (0,  1), (1,  1),
];

/// Priority-flood algorithm for depression filling
///
/// This algorithm fills terrain depressions below the ground_water_level.
/// Water naturally flows to the lowest point and fills up until it overflows.
pub fn compute_water_fill(
    heightmap: &HeightMap,
    ground_water_level: f32,
    min_depth: f32,
    min_area: usize,
) -> WaterFillResult {
    let width = heightmap.width;
    let depth = heightmap.depth;
    let mut water_levels = vec![f32::MAX; width * depth];
    let mut visited = vec![false; width * depth];
    let mut heap = BinaryHeap::new();

    // Step 1: Initialize boundary cells
    // Push all edge cells as starting points
    for x in 0..width {
        // Top and bottom edges
        for &z in &[0, depth - 1] {
            let idx = z * width + x;
            let h = heightmap.get_height(x, z);
            water_levels[idx] = h;
            visited[idx] = true;
            heap.push(FloodCell { x, z, height: h });
        }
    }
    for z in 1..depth - 1 {
        // Left and right edges
        for &x in &[0, width - 1] {
            let idx = z * width + x;
            let h = heightmap.get_height(x, z);
            water_levels[idx] = h;
            visited[idx] = true;
            heap.push(FloodCell { x, z, height: h });
        }
    }

    // Step 2: Priority-flood propagation
    while let Some(cell) = heap.pop() {
        for &(dx, dz) in &NEIGHBORS {
            let nx = cell.x as i32 + dx;
            let nz = cell.z as i32 + dz;

            if nx < 0 || nx >= width as i32 || nz < 0 || nz >= depth as i32 {
                continue;
            }

            let nx = nx as usize;
            let nz = nz as usize;
            let nidx = nz * width + nx;

            if visited[nidx] {
                continue;
            }
            visited[nidx] = true;

            let terrain_h = heightmap.get_height(nx, nz);
            let mut water_h = cell.height.max(terrain_h);

            // Clamp to ground water level
            if water_h > ground_water_level {
                water_h = terrain_h; // Above water table, no fill
            }

            water_levels[nidx] = water_h;
            heap.push(FloodCell { x: nx, z: nz, height: water_h });
        }
    }

    // Step 3: Identify water bodies (connected components where water_level > terrain_height)
    let water_bodies = identify_water_bodies(
        &water_levels,
        heightmap,
        min_depth,
        min_area,
    );

    WaterFillResult {
        water_levels,
        width,
        depth,
        water_bodies,
    }
}

/// Identify connected water bodies using flood-fill
fn identify_water_bodies(
    water_levels: &[f32],
    heightmap: &HeightMap,
    min_depth: f32,
    min_area: usize,
) -> Vec<ComputedWaterBody> {
    let width = heightmap.width;
    let depth = heightmap.depth;
    let mut visited = vec![false; width * depth];
    let mut bodies = Vec::new();
    let mut body_id = 0u32;

    // Debug: count cells with water
    let mut water_cells_count = 0;
    let mut max_depth = 0.0f32;
    for z in 0..depth {
        for x in 0..width {
            let idx = z * width + x;
            let terrain_h = heightmap.get_height(x, z);
            let water_h = water_levels[idx];
            let water_depth = water_h - terrain_h;
            if water_depth > 0.001 {
                water_cells_count += 1;
                max_depth = max_depth.max(water_depth);
            }
        }
    }
    log::info!("Water fill debug: {} cells have water, max depth={:.3}", water_cells_count, max_depth);

    // 4-connected neighbors for water body grouping (more conservative)
    let neighbors_4: [(i32, i32); 4] = [(0, -1), (-1, 0), (1, 0), (0, 1)];

    for z in 0..depth {
        for x in 0..width {
            let idx = z * width + x;
            if visited[idx] {
                continue;
            }

            let terrain_h = heightmap.get_height(x, z);
            let water_h = water_levels[idx];
            let water_depth = water_h - terrain_h;

            // Check if this cell has water
            if water_depth < min_depth {
                visited[idx] = true;
                continue;
            }

            // Found a water cell, flood-fill to find connected body
            let mut cells = Vec::new();
            let mut queue = VecDeque::new();
            queue.push_back((x, z));
            visited[idx] = true;

            let surface_level = water_h;
            let mut total_depth = 0.0f32;
            let mut min_x = x;
            let mut max_x = x;
            let mut min_z = z;
            let mut max_z = z;

            while let Some((cx, cz)) = queue.pop_front() {
                let cidx = cz * width + cx;
                let cell_terrain_h = heightmap.get_height(cx, cz);
                let cell_water_h = water_levels[cidx];
                let cell_depth = cell_water_h - cell_terrain_h;

                // Only include cells that are part of this water body
                // (same surface level and sufficient depth)
                if (cell_water_h - surface_level).abs() > 0.01 || cell_depth < min_depth {
                    continue;
                }

                cells.push((cx, cz));
                total_depth += cell_depth;
                min_x = min_x.min(cx);
                max_x = max_x.max(cx);
                min_z = min_z.min(cz);
                max_z = max_z.max(cz);

                // Check neighbors
                for &(dx, dz) in &neighbors_4 {
                    let nx = cx as i32 + dx;
                    let nz = cz as i32 + dz;

                    if nx < 0 || nx >= width as i32 || nz < 0 || nz >= depth as i32 {
                        continue;
                    }

                    let nx = nx as usize;
                    let nz = nz as usize;
                    let nidx = nz * width + nx;

                    if visited[nidx] {
                        continue;
                    }

                    let neighbor_terrain_h = heightmap.get_height(nx, nz);
                    let neighbor_water_h = water_levels[nidx];
                    let neighbor_depth = neighbor_water_h - neighbor_terrain_h;

                    // Include if same water level and has water
                    if (neighbor_water_h - surface_level).abs() < 0.01 && neighbor_depth >= min_depth {
                        visited[nidx] = true;
                        queue.push_back((nx, nz));
                    }
                }
            }

            // Check minimum area requirement
            if cells.len() < min_area {
                continue;
            }

            let average_depth = total_depth / cells.len() as f32;

            bodies.push(ComputedWaterBody {
                id: body_id,
                surface_level,
                cells,
                bounds_min: Vec3::new(min_x as f32, surface_level - average_depth, min_z as f32),
                bounds_max: Vec3::new(max_x as f32, surface_level, max_z as f32),
                average_depth,
                connected_to: Vec::new(),
                flow_direction: None,
                flow_speed: 0.0,
            });

            body_id += 1;
        }
    }

    // Detect overflow connections between bodies
    detect_overflow_connections(&mut bodies, water_levels, heightmap);

    bodies
}

/// Detect overflow connections between water bodies (rivers)
fn detect_overflow_connections(
    bodies: &mut [ComputedWaterBody],
    _water_levels: &[f32],
    heightmap: &HeightMap,
) {
    let width = heightmap.width;

    // Build a map from cell to body id
    let mut cell_to_body: HashMap<(usize, usize), usize> = HashMap::new();
    for (body_idx, body) in bodies.iter().enumerate() {
        for &(x, z) in &body.cells {
            cell_to_body.insert((x, z), body_idx);
        }
    }

    // Check for adjacent cells belonging to different bodies
    let mut connections: HashSet<(usize, usize)> = HashSet::new();

    for (body_idx, body) in bodies.iter().enumerate() {
        for &(x, z) in &body.cells {
            // Check 8-connected neighbors
            for &(dx, dz) in &NEIGHBORS {
                let nx = x as i32 + dx;
                let nz = z as i32 + dz;

                if nx < 0 || nx >= width as i32 || nz < 0 || nz >= heightmap.depth as i32 {
                    continue;
                }

                let nx = nx as usize;
                let nz = nz as usize;

                if let Some(&other_idx) = cell_to_body.get(&(nx, nz)) {
                    if other_idx != body_idx {
                        // Found connection between two bodies
                        let (a, b) = if body_idx < other_idx {
                            (body_idx, other_idx)
                        } else {
                            (other_idx, body_idx)
                        };
                        connections.insert((a, b));
                    }
                }
            }
        }
    }

    // Add connections and compute flow direction
    for (a, b) in connections {
        // Extract data we need before mutating
        let surface_a = bodies[a].surface_level;
        let surface_b = bodies[b].surface_level;
        let bounds_a_min = bodies[a].bounds_min;
        let bounds_a_max = bodies[a].bounds_max;
        let bounds_b_min = bodies[b].bounds_min;
        let bounds_b_max = bodies[b].bounds_max;
        let id_a = bodies[a].id;
        let id_b = bodies[b].id;

        // Determine flow direction (higher to lower)
        if surface_a > surface_b {
            // Flow from a to b
            let center_a = (bounds_a_min + bounds_a_max) * 0.5;
            let center_b = (bounds_b_min + bounds_b_max) * 0.5;
            let dir = (center_b - center_a).normalize();
            let height_diff = surface_a - surface_b;

            bodies[a].connected_to.push(id_b);
            bodies[a].flow_direction = Some([dir.x, dir.z]);
            bodies[a].flow_speed = (height_diff * 2.0).min(5.0);
        } else if surface_b > surface_a {
            // Flow from b to a
            let center_a = (bounds_a_min + bounds_a_max) * 0.5;
            let center_b = (bounds_b_min + bounds_b_max) * 0.5;
            let dir = (center_a - center_b).normalize();
            let height_diff = surface_b - surface_a;

            bodies[b].connected_to.push(id_a);
            bodies[b].flow_direction = Some([dir.x, dir.z]);
            bodies[b].flow_speed = (height_diff * 2.0).min(5.0);
        }
        // If same level, they're part of the same lake (no flow)
    }
}

/// Generate a water mesh for a water body that conforms to terrain boundaries
pub fn generate_water_mesh(
    water_body: &ComputedWaterBody,
    _heightmap: &HeightMap,
    config: &TerrainConfig,
) -> Mesh {
    let cell_size = config.scale / config.width as f32;
    let offset_x = -config.scale * 0.5;
    let offset_z = -config.scale * 0.5;

    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut vertex_map: HashMap<(usize, usize), u32> = HashMap::new();

    // Create a set of water cells for quick lookup
    let water_cells: HashSet<(usize, usize)> = water_body.cells.iter().cloned().collect();

    // Create vertices for each water cell
    for &(gx, gz) in &water_body.cells {
        let world_x = offset_x + gx as f32 * cell_size;
        let world_z = offset_z + gz as f32 * cell_size;

        let vertex_idx = vertices.len() as u32;
        vertex_map.insert((gx, gz), vertex_idx);

        // UV based on world position for seamless tiling
        let u = (world_x / config.scale + 0.5) * 4.0; // Tile 4x
        let v = (world_z / config.scale + 0.5) * 4.0;

        vertices.push(
            Vertex::new(Vec3::new(world_x, water_body.surface_level, world_z))
                .with_normal(Vec3::Y)
                .with_tex_coord(Vec2::new(u, v))
                .with_color(Vec3::ONE),
        );
    }

    // Create triangles for adjacent cells
    for &(gx, gz) in &water_body.cells {
        // Check if we can form quads with neighbors
        let has_right = water_cells.contains(&(gx + 1, gz));
        let has_bottom = water_cells.contains(&(gx, gz + 1));
        let has_diag = water_cells.contains(&(gx + 1, gz + 1));

        if has_right && has_bottom && has_diag {
            let v00 = vertex_map[&(gx, gz)];
            let v10 = vertex_map[&(gx + 1, gz)];
            let v01 = vertex_map[&(gx, gz + 1)];
            let v11 = vertex_map[&(gx + 1, gz + 1)];

            // Two triangles per quad (counter-clockwise winding)
            indices.extend_from_slice(&[v00, v10, v01, v10, v11, v01]);
        }
    }

    // Calculate tangents for the water mesh
    let mut mesh = Mesh::new(
        format!("terrain_water_{}", water_body.id),
        vertices,
        indices,
    );
    mesh.calculate_tangents();
    mesh
}
