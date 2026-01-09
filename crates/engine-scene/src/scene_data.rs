// Serializable scene format for saving and loading scenes

use crate::components::*;
use crate::entity::EntityId;
use crate::transform::Transform;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// All component types that can be serialized
/// Note: Physics components (RigidBody, Collider) are serialized as generic data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SerializedComponent {
    MeshRenderer(MeshRenderer),
    Camera(Camera),
    Light(Light),
    ParticleEmitter(ParticleEmitter),
    // Generic component data for extensibility (e.g., physics components)
    Generic {
        component_type: String,
        data: String,  // RON-serialized component data
    },
}

/// Serializable entity data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedEntity {
    pub id: EntityId,
    pub name: String,
    pub transform: Transform,
    pub parent: Option<EntityId>,
    pub children: Vec<EntityId>,
    pub components: Vec<SerializedComponent>,
}

/// Serializable scene data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedScene {
    pub name: String,
    pub entities: HashMap<EntityId, SerializedEntity>,
    pub next_id: u64,
    pub root_entities: Vec<EntityId>,
}
