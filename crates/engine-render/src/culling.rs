// Culling system - manages visibility testing for renderables

use glam::{Mat4, Vec3};
use std::collections::HashMap;

use crate::frustum::{Frustum, AABB};

/// Unique identifier for a renderable object
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RenderableId(pub u64);

/// Renderable object with bounding volume
#[derive(Debug, Clone)]
pub struct Renderable {
    /// Unique identifier
    pub id: RenderableId,
    /// Bounding box in local space
    pub local_bounds: AABB,
    /// World transform matrix
    pub transform: Mat4,
    /// Whether this object casts shadows
    pub casts_shadow: bool,
    /// Whether frustum culling is enabled for this object
    pub cull_enabled: bool,
}

impl Renderable {
    /// Create a new renderable
    pub fn new(id: RenderableId, local_bounds: AABB, transform: Mat4) -> Self {
        Self {
            id,
            local_bounds,
            transform,
            casts_shadow: true,
            cull_enabled: true,
        }
    }

    /// Get world-space bounding box
    pub fn world_bounds(&self) -> AABB {
        self.local_bounds.transform(self.transform)
    }

    /// Get world-space center
    pub fn world_center(&self) -> Vec3 {
        self.world_bounds().center()
    }

    /// Get bounding sphere radius in world space
    pub fn world_bounding_sphere_radius(&self) -> f32 {
        self.world_bounds().bounding_sphere_radius()
    }

    /// Disable frustum culling for this object (always render)
    pub fn with_culling_disabled(mut self) -> Self {
        self.cull_enabled = false;
        self
    }

    /// Set shadow casting
    pub fn with_shadow_casting(mut self, casts_shadow: bool) -> Self {
        self.casts_shadow = casts_shadow;
        self
    }
}

/// Culling system manages visibility testing
pub struct CullingSystem {
    /// All registered renderables
    renderables: HashMap<RenderableId, Renderable>,
    /// Next available ID
    next_id: u64,
}

impl CullingSystem {
    /// Create a new culling system
    pub fn new() -> Self {
        Self {
            renderables: HashMap::new(),
            next_id: 1,
        }
    }

    /// Register a new renderable object
    pub fn register(&mut self, local_bounds: AABB, transform: Mat4) -> RenderableId {
        let id = RenderableId(self.next_id);
        self.next_id += 1;

        let renderable = Renderable::new(id, local_bounds, transform);
        self.renderables.insert(id, renderable);
        id
    }

    /// Register a renderable with specific settings
    pub fn register_with_config(&mut self, renderable: Renderable) -> RenderableId {
        let id = renderable.id;
        self.renderables.insert(id, renderable);
        id
    }

    /// Unregister a renderable
    pub fn unregister(&mut self, id: RenderableId) -> bool {
        self.renderables.remove(&id).is_some()
    }

    /// Update a renderable's transform
    pub fn update_transform(&mut self, id: RenderableId, transform: Mat4) -> bool {
        if let Some(renderable) = self.renderables.get_mut(&id) {
            renderable.transform = transform;
            true
        } else {
            false
        }
    }

    /// Update a renderable's local bounds
    pub fn update_bounds(&mut self, id: RenderableId, local_bounds: AABB) -> bool {
        if let Some(renderable) = self.renderables.get_mut(&id) {
            renderable.local_bounds = local_bounds;
            true
        } else {
            false
        }
    }

    /// Get a renderable
    pub fn get(&self, id: RenderableId) -> Option<&Renderable> {
        self.renderables.get(&id)
    }

    /// Get mutable renderable
    pub fn get_mut(&mut self, id: RenderableId) -> Option<&mut Renderable> {
        self.renderables.get_mut(&id)
    }

    /// Perform frustum culling and return visible object IDs
    pub fn cull(&self, frustum: &Frustum) -> Vec<RenderableId> {
        let mut visible = Vec::new();

        for renderable in self.renderables.values() {
            // Skip culling if disabled
            if !renderable.cull_enabled {
                visible.push(renderable.id);
                continue;
            }

            // Test against frustum
            if self.is_visible(renderable, frustum) {
                visible.push(renderable.id);
            }
        }

        visible
    }

    /// Cull and return shadow casters
    pub fn cull_shadow_casters(&self, frustum: &Frustum) -> Vec<RenderableId> {
        let mut visible = Vec::new();

        for renderable in self.renderables.values() {
            if !renderable.casts_shadow {
                continue;
            }

            // Skip culling if disabled
            if !renderable.cull_enabled {
                visible.push(renderable.id);
                continue;
            }

            if self.is_visible(renderable, frustum) {
                visible.push(renderable.id);
            }
        }

        visible
    }

    /// Test if a renderable is visible in the frustum
    fn is_visible(&self, renderable: &Renderable, frustum: &Frustum) -> bool {
        let world_bounds = renderable.world_bounds();
        frustum.contains_aabb(world_bounds.min, world_bounds.max)
    }

    /// Get statistics
    pub fn stats(&self) -> CullingStats {
        CullingStats {
            total_renderables: self.renderables.len(),
        }
    }

    /// Clear all renderables
    pub fn clear(&mut self) {
        self.renderables.clear();
        self.next_id = 1;
    }

    /// Get all renderable IDs
    pub fn all_ids(&self) -> Vec<RenderableId> {
        self.renderables.keys().copied().collect()
    }

    /// Get number of renderables
    pub fn count(&self) -> usize {
        self.renderables.len()
    }
}

impl Default for CullingSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Culling statistics
#[derive(Debug, Clone, Copy)]
pub struct CullingStats {
    /// Total number of registered renderables
    pub total_renderables: usize,
}

/// Visibility result from culling
#[derive(Debug, Clone)]
pub struct VisibilityResult {
    /// IDs of visible objects for main camera
    pub visible: Vec<RenderableId>,
    /// IDs of visible shadow casters
    pub shadow_casters: Vec<RenderableId>,
    /// Number of objects culled
    pub culled_count: usize,
}

impl VisibilityResult {
    /// Create from culling results
    pub fn new(visible: Vec<RenderableId>, shadow_casters: Vec<RenderableId>, total: usize) -> Self {
        Self {
            culled_count: total.saturating_sub(visible.len()),
            visible,
            shadow_casters,
        }
    }

    /// Get culling efficiency (percentage culled)
    pub fn cull_efficiency(&self) -> f32 {
        let total = self.visible.len() + self.culled_count;
        if total == 0 {
            0.0
        } else {
            (self.culled_count as f32 / total as f32) * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_culling_system_register() {
        let mut system = CullingSystem::new();
        let bounds = AABB::new(Vec3::splat(-1.0), Vec3::splat(1.0));
        let transform = Mat4::IDENTITY;

        let id = system.register(bounds, transform);
        assert_eq!(system.count(), 1);
        assert!(system.get(id).is_some());
    }

    #[test]
    fn test_culling_system_unregister() {
        let mut system = CullingSystem::new();
        let bounds = AABB::new(Vec3::splat(-1.0), Vec3::splat(1.0));
        let id = system.register(bounds, Mat4::IDENTITY);

        assert!(system.unregister(id));
        assert_eq!(system.count(), 0);
        assert!(system.get(id).is_none());
    }

    #[test]
    fn test_culling_system_update() {
        let mut system = CullingSystem::new();
        let bounds = AABB::new(Vec3::splat(-1.0), Vec3::splat(1.0));
        let id = system.register(bounds, Mat4::IDENTITY);

        let new_transform = Mat4::from_translation(Vec3::new(10.0, 0.0, 0.0));
        assert!(system.update_transform(id, new_transform));

        let renderable = system.get(id).unwrap();
        assert_eq!(renderable.transform, new_transform);
    }

    #[test]
    fn test_frustum_culling() {
        let mut system = CullingSystem::new();

        // Create orthographic frustum
        let view_proj = Mat4::orthographic_rh(-10.0, 10.0, -10.0, 10.0, 0.1, 100.0);
        let frustum = Frustum::from_view_projection(view_proj);

        // Object inside frustum
        let bounds1 = AABB::new(Vec3::splat(-1.0), Vec3::splat(1.0));
        let id1 = system.register(bounds1, Mat4::from_translation(Vec3::new(0.0, 0.0, -10.0)));

        // Object outside frustum (too far)
        let bounds2 = AABB::new(Vec3::splat(-1.0), Vec3::splat(1.0));
        let id2 = system.register(bounds2, Mat4::from_translation(Vec3::new(0.0, 0.0, -200.0)));

        // Object outside frustum (to the side)
        let bounds3 = AABB::new(Vec3::splat(-1.0), Vec3::splat(1.0));
        let id3 = system.register(bounds3, Mat4::from_translation(Vec3::new(50.0, 0.0, -10.0)));

        let visible = system.cull(&frustum);

        // Only id1 should be visible
        assert_eq!(visible.len(), 1);
        assert!(visible.contains(&id1));
        assert!(!visible.contains(&id2));
        assert!(!visible.contains(&id3));
    }

    #[test]
    fn test_culling_disabled() {
        let mut system = CullingSystem::new();
        let view_proj = Mat4::orthographic_rh(-10.0, 10.0, -10.0, 10.0, 0.1, 100.0);
        let frustum = Frustum::from_view_projection(view_proj);

        // Object outside frustum but with culling disabled
        let bounds = AABB::new(Vec3::splat(-1.0), Vec3::splat(1.0));
        let renderable = Renderable::new(
            RenderableId(1),
            bounds,
            Mat4::from_translation(Vec3::new(0.0, 0.0, -200.0)),
        )
        .with_culling_disabled();

        system.register_with_config(renderable);

        let visible = system.cull(&frustum);
        assert_eq!(visible.len(), 1); // Should be visible despite being outside frustum
    }

    #[test]
    fn test_shadow_caster_culling() {
        let mut system = CullingSystem::new();
        let view_proj = Mat4::orthographic_rh(-10.0, 10.0, -10.0, 10.0, 0.1, 100.0);
        let frustum = Frustum::from_view_projection(view_proj);

        let bounds = AABB::new(Vec3::splat(-1.0), Vec3::splat(1.0));

        // Shadow caster
        let r1 = Renderable::new(RenderableId(1), bounds, Mat4::from_translation(Vec3::new(0.0, 0.0, -10.0)))
            .with_shadow_casting(true);
        system.register_with_config(r1);

        // Non-shadow caster
        let r2 = Renderable::new(RenderableId(2), bounds, Mat4::from_translation(Vec3::new(2.0, 0.0, -10.0)))
            .with_shadow_casting(false);
        system.register_with_config(r2);

        let shadow_casters = system.cull_shadow_casters(&frustum);
        assert_eq!(shadow_casters.len(), 1);
        assert!(shadow_casters.contains(&RenderableId(1)));
    }

    #[test]
    fn test_visibility_result() {
        let visible = vec![RenderableId(1), RenderableId(2)];
        let shadow_casters = vec![RenderableId(1)];
        let result = VisibilityResult::new(visible, shadow_casters, 10);

        assert_eq!(result.visible.len(), 2);
        assert_eq!(result.culled_count, 8);
        assert_eq!(result.cull_efficiency(), 80.0);
    }
}
