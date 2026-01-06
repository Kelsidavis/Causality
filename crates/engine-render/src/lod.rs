// Level of Detail (LOD) system for performance optimization

use glam::Vec3;
use crate::gpu_mesh::MeshHandle;

/// LOD level configuration
#[derive(Debug, Clone)]
pub struct LodLevel {
    /// Mesh to use at this LOD level
    pub mesh: MeshHandle,
    /// Minimum distance (squared) for this LOD level
    /// Using squared distance avoids sqrt calculation
    pub min_distance_squared: f32,
}

impl LodLevel {
    /// Create a new LOD level
    pub fn new(mesh: MeshHandle, min_distance: f32) -> Self {
        Self {
            mesh,
            min_distance_squared: min_distance * min_distance,
        }
    }

    /// Check if this LOD level should be used at the given distance
    pub fn is_active(&self, distance_squared: f32) -> bool {
        distance_squared >= self.min_distance_squared
    }
}

/// LOD configuration for an entity
#[derive(Debug, Clone)]
pub struct LodConfig {
    /// LOD levels, sorted from highest detail (LOD0) to lowest detail
    /// First level (LOD0) should have min_distance = 0.0
    pub levels: Vec<LodLevel>,
    /// Whether LOD is enabled
    pub enabled: bool,
}

impl LodConfig {
    /// Create a new LOD configuration
    pub fn new() -> Self {
        Self {
            levels: Vec::new(),
            enabled: true,
        }
    }

    /// Add a LOD level
    pub fn add_level(mut self, mesh: MeshHandle, min_distance: f32) -> Self {
        self.levels.push(LodLevel::new(mesh, min_distance));
        self.levels.sort_by(|a, b| {
            a.min_distance_squared
                .partial_cmp(&b.min_distance_squared)
                .unwrap()
        });
        self
    }

    /// Create a simple 2-level LOD (high detail, low detail)
    pub fn two_level(high_detail: MeshHandle, low_detail: MeshHandle, switch_distance: f32) -> Self {
        Self::new()
            .add_level(high_detail, 0.0) // LOD0 - close up
            .add_level(low_detail, switch_distance) // LOD1 - far away
    }

    /// Create a 3-level LOD (high, medium, low detail)
    pub fn three_level(
        high_detail: MeshHandle,
        medium_detail: MeshHandle,
        low_detail: MeshHandle,
        medium_distance: f32,
        low_distance: f32,
    ) -> Self {
        Self::new()
            .add_level(high_detail, 0.0) // LOD0
            .add_level(medium_detail, medium_distance) // LOD1
            .add_level(low_detail, low_distance) // LOD2
    }

    /// Select the appropriate mesh based on distance from camera
    /// Returns the mesh handle and LOD level index
    pub fn select_lod(&self, distance_squared: f32) -> Option<(MeshHandle, usize)> {
        if !self.enabled || self.levels.is_empty() {
            return None;
        }

        // Start from highest LOD and find the first one that matches distance
        // Levels are sorted by distance, so iterate in reverse
        for (i, level) in self.levels.iter().enumerate().rev() {
            if distance_squared >= level.min_distance_squared {
                return Some((level.mesh, i));
            }
        }

        // If nothing matched (shouldn't happen if LOD0 has min_distance = 0.0),
        // return the highest detail level
        self.levels
            .first()
            .map(|level| (level.mesh, 0))
    }

    /// Get the number of LOD levels
    pub fn level_count(&self) -> usize {
        self.levels.len()
    }

    /// Enable LOD
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Disable LOD (always use highest detail)
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Set enabled state
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

impl Default for LodConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// LOD bias - global multiplier for LOD distances
/// Values > 1.0 = use higher detail (better quality, worse performance)
/// Values < 1.0 = use lower detail (worse quality, better performance)
pub struct LodBias {
    bias: f32,
}

impl LodBias {
    /// Create with default bias (1.0)
    pub fn new() -> Self {
        Self { bias: 1.0 }
    }

    /// Create with custom bias
    pub fn with_bias(bias: f32) -> Self {
        Self {
            bias: bias.max(0.1), // Prevent zero/negative bias
        }
    }

    /// Set the bias
    pub fn set_bias(&mut self, bias: f32) {
        self.bias = bias.max(0.1);
    }

    /// Get the bias
    pub fn bias(&self) -> f32 {
        self.bias
    }

    /// Apply bias to a distance (squared)
    pub fn apply(&self, distance_squared: f32) -> f32 {
        distance_squared / (self.bias * self.bias)
    }
}

impl Default for LodBias {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper to calculate distance squared from camera to position
#[inline]
pub fn distance_squared(camera_pos: Vec3, object_pos: Vec3) -> f32 {
    (camera_pos - object_pos).length_squared()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lod_level() {
        let level = LodLevel::new(MeshHandle(0), 10.0);
        assert_eq!(level.min_distance_squared, 100.0);
        assert!(!level.is_active(50.0)); // 50 < 100
        assert!(level.is_active(100.0)); // 100 == 100
        assert!(level.is_active(150.0)); // 150 > 100
    }

    #[test]
    fn test_lod_selection() {
        let lod = LodConfig::three_level(
            MeshHandle(0), // High detail
            MeshHandle(1), // Medium detail
            MeshHandle(2), // Low detail
            10.0,          // Medium starts at 10m
            50.0,          // Low starts at 50m
        );

        // Close up - should use LOD0 (high detail)
        let (mesh, level) = lod.select_lod(0.0).unwrap();
        assert_eq!(mesh, MeshHandle(0));
        assert_eq!(level, 0);

        // Medium distance - should use LOD1 (medium detail)
        let (mesh, level) = lod.select_lod(15.0 * 15.0).unwrap();
        assert_eq!(mesh, MeshHandle(1));
        assert_eq!(level, 1);

        // Far distance - should use LOD2 (low detail)
        let (mesh, level) = lod.select_lod(100.0 * 100.0).unwrap();
        assert_eq!(mesh, MeshHandle(2));
        assert_eq!(level, 2);
    }

    #[test]
    fn test_lod_bias() {
        let mut bias = LodBias::new();
        assert_eq!(bias.bias(), 1.0);

        // Higher bias = use higher detail (multiply effective distance)
        bias.set_bias(2.0);
        let dist_sq = 100.0;
        assert_eq!(bias.apply(dist_sq), 25.0); // 100 / (2*2) = 25

        // Lower bias = use lower detail (divide effective distance)
        bias.set_bias(0.5);
        assert_eq!(bias.apply(dist_sq), 400.0); // 100 / (0.5*0.5) = 400
    }

    #[test]
    fn test_distance_calculation() {
        let camera_pos = Vec3::new(0.0, 0.0, 0.0);
        let object_pos = Vec3::new(3.0, 4.0, 0.0);
        let dist_sq = distance_squared(camera_pos, object_pos);
        assert_eq!(dist_sq, 25.0); // 3^2 + 4^2 = 25
    }

    #[test]
    fn test_disabled_lod() {
        let mut lod = LodConfig::two_level(MeshHandle(0), MeshHandle(1), 10.0);
        lod.disable();

        // When disabled, should return None
        assert!(lod.select_lod(100.0).is_none());
    }
}
