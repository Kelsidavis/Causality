// Raycasting for physics queries

use glam::Vec3;
use rapier3d::prelude::*;
use engine_scene::entity::EntityId;
use std::collections::HashMap;

/// Result of a raycast query
#[derive(Debug, Clone)]
pub struct RaycastHit {
    /// The entity that was hit
    pub entity_id: EntityId,
    /// Point of intersection in world space
    pub point: Vec3,
    /// Normal at the hit point
    pub normal: Vec3,
    /// Distance from ray origin to hit point
    pub distance: f32,
}

/// Raycast query interface
pub struct RaycastQuery {
    /// Ray origin
    pub origin: Vec3,
    /// Ray direction (should be normalized)
    pub direction: Vec3,
    /// Maximum distance to check
    pub max_distance: f32,
    /// Whether to hit triggers/sensors
    pub hit_triggers: bool,
}

impl RaycastQuery {
    pub fn new(origin: Vec3, direction: Vec3, max_distance: f32) -> Self {
        Self {
            origin,
            direction: direction.normalize(),
            max_distance,
            hit_triggers: false,
        }
    }

    pub fn with_triggers(mut self, hit_triggers: bool) -> Self {
        self.hit_triggers = hit_triggers;
        self
    }
}

impl super::PhysicsWorld {
    /// Cast a ray and return the first hit
    pub fn raycast(&self, query: &RaycastQuery) -> Option<RaycastHit> {
        let ray = Ray::new(
            point![query.origin.x, query.origin.y, query.origin.z],
            vector![query.direction.x, query.direction.y, query.direction.z],
        );

        let filter = QueryFilter::default();

        if let Some((collider_handle, intersection)) = self.query_pipeline.cast_ray_and_get_normal(
            &self.rigid_body_set,
            &self.collider_set,
            &ray,
            query.max_distance,
            true,
            filter,
        ) {
            let hit_point = ray.point_at(intersection.time_of_impact);

            // Get entity ID from collider's parent rigid body
            let entity_id = self.collider_set
                .get(collider_handle)
                .and_then(|collider| collider.parent())
                .and_then(|body_handle| self.get_entity_id(body_handle))
                .unwrap_or(EntityId(0)); // Fallback if not found

            Some(RaycastHit {
                entity_id,
                point: Vec3::new(hit_point.x, hit_point.y, hit_point.z),
                normal: Vec3::new(intersection.normal.x, intersection.normal.y, intersection.normal.z),
                distance: intersection.time_of_impact,
            })
        } else {
            None
        }
    }

    /// Cast a ray and return all hits along the ray
    pub fn raycast_all(&self, query: &RaycastQuery) -> Vec<RaycastHit> {
        let ray = Ray::new(
            point![query.origin.x, query.origin.y, query.origin.z],
            vector![query.direction.x, query.direction.y, query.direction.z],
        );

        let filter = QueryFilter::default();

        let mut hits = Vec::new();

        self.query_pipeline.intersections_with_ray(
            &self.rigid_body_set,
            &self.collider_set,
            &ray,
            query.max_distance,
            false, // solid - continue through all objects
            filter,
            |collider_handle, intersection: rapier3d::prelude::RayIntersection| {
                let hit_point = ray.point_at(intersection.time_of_impact);

                // Get entity ID from collider's parent rigid body
                let entity_id = self.collider_set
                    .get(collider_handle)
                    .and_then(|collider| collider.parent())
                    .and_then(|body_handle| self.get_entity_id(body_handle))
                    .unwrap_or(EntityId(0)); // Fallback if not found

                hits.push(RaycastHit {
                    entity_id,
                    point: Vec3::new(hit_point.x, hit_point.y, hit_point.z),
                    normal: Vec3::new(intersection.normal.x, intersection.normal.y, intersection.normal.z),
                    distance: intersection.time_of_impact,
                });

                true // Continue checking
            },
        );

        // Sort by distance
        hits.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());
        hits
    }

    /// Check if a ray intersects anything (no detailed hit info)
    pub fn raycast_any(&self, query: &RaycastQuery) -> bool {
        let ray = Ray::new(
            point![query.origin.x, query.origin.y, query.origin.z],
            vector![query.direction.x, query.direction.y, query.direction.z],
        );

        let filter = QueryFilter::default();

        self.query_pipeline
            .cast_ray(
                &self.rigid_body_set,
                &self.collider_set,
                &ray,
                query.max_distance,
                false,
                filter,
            )
            .is_some()
    }
}
