// Physics world - wraps Rapier3D

use glam::{Quat, Vec3};
use rapier3d::prelude::*;
use rapier3d::na::{Quaternion, UnitQuaternion};
use std::collections::HashMap;

use engine_scene::entity::EntityId;

/// Physics world - manages all physics simulation
pub struct PhysicsWorld {
    pub gravity: Vec3,
    pub integration_parameters: IntegrationParameters,
    pub physics_pipeline: PhysicsPipeline,
    pub island_manager: IslandManager,
    pub broad_phase: DefaultBroadPhase,
    pub narrow_phase: NarrowPhase,
    pub rigid_body_set: RigidBodySet,
    pub collider_set: ColliderSet,
    pub impulse_joint_set: ImpulseJointSet,
    pub multibody_joint_set: MultibodyJointSet,
    pub ccd_solver: CCDSolver,
    pub query_pipeline: QueryPipeline,

    // Mapping between entity IDs and Rapier handles
    entity_to_body: HashMap<EntityId, RigidBodyHandle>,
    body_to_entity: HashMap<RigidBodyHandle, EntityId>,
}

impl PhysicsWorld {
    pub fn new(gravity: Vec3) -> Self {
        Self {
            gravity,
            integration_parameters: IntegrationParameters::default(),
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: DefaultBroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            rigid_body_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            query_pipeline: QueryPipeline::new(),
            entity_to_body: HashMap::new(),
            body_to_entity: HashMap::new(),
        }
    }

    /// Step the physics simulation
    pub fn step(&mut self, delta_time: f32) {
        self.integration_parameters.dt = delta_time;

        let gravity = vector![self.gravity.x, self.gravity.y, self.gravity.z];

        self.physics_pipeline.step(
            &gravity,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            Some(&mut self.query_pipeline),
            &(),
            &(),
        );
    }

    /// Create a rigid body and associate it with an entity
    pub fn create_rigid_body(&mut self, entity_id: EntityId, rigid_body: RigidBody) -> RigidBodyHandle {
        let handle = self.rigid_body_set.insert(rigid_body);
        self.entity_to_body.insert(entity_id, handle);
        self.body_to_entity.insert(handle, entity_id);
        handle
    }

    /// Remove a rigid body
    pub fn remove_rigid_body(&mut self, entity_id: EntityId) {
        if let Some(handle) = self.entity_to_body.remove(&entity_id) {
            self.body_to_entity.remove(&handle);
            self.rigid_body_set.remove(
                handle,
                &mut self.island_manager,
                &mut self.collider_set,
                &mut self.impulse_joint_set,
                &mut self.multibody_joint_set,
                true,
            );
        }
    }

    /// Get rigid body handle for an entity
    pub fn get_body_handle(&self, entity_id: EntityId) -> Option<RigidBodyHandle> {
        self.entity_to_body.get(&entity_id).copied()
    }

    /// Get entity ID for a rigid body handle
    pub fn get_entity_id(&self, handle: RigidBodyHandle) -> Option<EntityId> {
        self.body_to_entity.get(&handle).copied()
    }

    /// Get a rigid body reference
    pub fn get_rigid_body(&self, handle: RigidBodyHandle) -> Option<&RigidBody> {
        self.rigid_body_set.get(handle)
    }

    /// Get a mutable rigid body reference
    pub fn get_rigid_body_mut(&mut self, handle: RigidBodyHandle) -> Option<&mut RigidBody> {
        self.rigid_body_set.get_mut(handle)
    }

    /// Create a collider attached to a rigid body
    pub fn create_collider(&mut self, body_handle: RigidBodyHandle, collider: Collider) -> ColliderHandle {
        self.collider_set.insert_with_parent(collider, body_handle, &mut self.rigid_body_set)
    }

    /// Remove a collider
    pub fn remove_collider(&mut self, handle: ColliderHandle) {
        self.collider_set.remove(handle, &mut self.island_manager, &mut self.rigid_body_set, true);
    }
}

impl Default for PhysicsWorld {
    fn default() -> Self {
        Self::new(Vec3::new(0.0, -9.81, 0.0))
    }
}

/// Helper function to convert glam Vec3 to Rapier vector
pub fn to_rapier_vec(v: Vec3) -> Vector<f32> {
    vector![v.x, v.y, v.z]
}

/// Helper function to convert Rapier vector to glam Vec3
pub fn from_rapier_vec(v: Vector<f32>) -> Vec3 {
    Vec3::new(v.x, v.y, v.z)
}

/// Helper function to convert glam Quat to Rapier quaternion
pub fn to_rapier_quat(q: Quat) -> UnitQuaternion<f32> {
    UnitQuaternion::new_normalize(Quaternion::new(q.w, q.x, q.y, q.z))
}

/// Helper function to convert Rapier quaternion to glam Quat
pub fn from_rapier_quat(q: UnitQuaternion<f32>) -> Quat {
    Quat::from_xyzw(q.i, q.j, q.k, q.w)
}
