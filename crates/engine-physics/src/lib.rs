// Engine Physics - Rapier3D integration

pub mod character;
pub mod components;
pub mod joints;
pub mod layers;
pub mod raycast;
pub mod sync;
pub mod world;

pub use character::CharacterController;
pub use components::{Collider, ColliderShape, RigidBody, RigidBodyType};
pub use joints::{JointConfig, JointHandle, JointManager, JointType};
pub use layers::CollisionGroups;
pub mod collision_layers {
    pub use crate::layers::layers::*;
}
pub use raycast::{RaycastHit, RaycastQuery};
pub use sync::PhysicsSync;
pub use world::{from_rapier_quat, from_rapier_vec, to_rapier_quat, to_rapier_vec, PhysicsWorld};
