// Physics synchronization - sync physics world to scene transforms

use crate::components::{Collider, ColliderShape, RigidBody, RigidBodyType};
use crate::world::{from_rapier_quat, from_rapier_vec, to_rapier_quat, to_rapier_vec, PhysicsWorld};
use anyhow::Result;
use engine_scene::scene::Scene;
use rapier3d::prelude::*;

/// Sync system - manages synchronization between physics and scene
pub struct PhysicsSync;

impl PhysicsSync {
    /// Initialize physics bodies for entities that have RigidBody and Collider components
    pub fn initialize_physics(physics_world: &mut PhysicsWorld, scene: &Scene) -> Result<()> {
        for entity in scene.entities() {
            // Skip if already has physics body
            if physics_world.get_body_handle(entity.id).is_some() {
                continue;
            }

            // Check if entity has both RigidBody and Collider components
            if let (Some(rb_component), Some(col_component)) = (
                entity.get_component::<RigidBody>(),
                entity.get_component::<Collider>(),
            ) {
                // Create Rapier rigid body
                let position = to_rapier_vec(entity.transform.position);
                let rotation = to_rapier_quat(entity.transform.rotation);

                let rapier_body = match rb_component.body_type {
                    RigidBodyType::Dynamic => RigidBodyBuilder::dynamic()
                        .position(Isometry::from_parts(position.into(), rotation))
                        .linvel(to_rapier_vec(rb_component.linear_velocity))
                        .angvel(to_rapier_vec(rb_component.angular_velocity).into())
                        .linear_damping(rb_component.linear_damping)
                        .angular_damping(rb_component.angular_damping)
                        .can_sleep(rb_component.can_sleep)
                        .ccd_enabled(rb_component.ccd_enabled)
                        .build(),
                    RigidBodyType::Kinematic => RigidBodyBuilder::kinematic_position_based()
                        .position(Isometry::from_parts(position.into(), rotation))
                        .build(),
                    RigidBodyType::Static => RigidBodyBuilder::fixed()
                        .position(Isometry::from_parts(position.into(), rotation))
                        .build(),
                };

                let body_handle = physics_world.create_rigid_body(entity.id, rapier_body);

                // Create Rapier collider
                let rapier_collider = match &col_component.shape {
                    ColliderShape::Box { half_extents } => {
                        let he = to_rapier_vec(*half_extents);
                        ColliderBuilder::cuboid(he.x, he.y, he.z)
                    }
                    ColliderShape::Sphere { radius } => ColliderBuilder::ball(*radius),
                    ColliderShape::Capsule { half_height, radius } => {
                        ColliderBuilder::capsule_y(*half_height, *radius)
                    }
                    ColliderShape::Cylinder { half_height, radius } => {
                        ColliderBuilder::cylinder(*half_height, *radius)
                    }
                }
                .friction(col_component.friction)
                .restitution(col_component.restitution)
                .density(col_component.density)
                .sensor(col_component.is_sensor)
                .build();

                physics_world.create_collider(body_handle, rapier_collider);
            }
        }

        Ok(())
    }

    /// Sync physics world state back to scene transforms (for dynamic bodies)
    pub fn sync_to_scene(physics_world: &PhysicsWorld, scene: &mut Scene) -> Result<()> {
        // Collect entity IDs first to avoid borrow checker issues
        let entity_ids: Vec<_> = scene.entities().map(|e| e.id).collect();

        for entity_id in entity_ids {
            // Get the physics body handle for this entity
            if let Some(body_handle) = physics_world.get_body_handle(entity_id) {
                if let Some(rapier_body) = physics_world.get_rigid_body(body_handle) {
                    // Only sync dynamic bodies (static and kinematic are controlled by the scene)
                    if rapier_body.is_dynamic() {
                        let position = rapier_body.translation();
                        let rotation = rapier_body.rotation();

                        // Update the entity's transform
                        if let Some(entity_mut) = scene.get_entity_mut(entity_id) {
                            entity_mut.transform.position = from_rapier_vec(*position);
                            entity_mut.transform.rotation = from_rapier_quat(*rotation);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Sync scene transforms to physics (for kinematic bodies)
    pub fn sync_from_scene(physics_world: &mut PhysicsWorld, scene: &Scene) -> Result<()> {
        for entity in scene.entities() {
            if let Some(body_handle) = physics_world.get_body_handle(entity.id) {
                if let Some(rapier_body) = physics_world.get_rigid_body_mut(body_handle) {
                    // Only sync kinematic bodies (they're controlled by the scene)
                    if rapier_body.is_kinematic() {
                        let position = to_rapier_vec(entity.transform.position);
                        let rotation = to_rapier_quat(entity.transform.rotation);
                        rapier_body.set_next_kinematic_position(Isometry::from_parts(
                            position.into(),
                            rotation,
                        ));
                    }
                }
            }
        }

        Ok(())
    }
}
