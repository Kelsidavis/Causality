// Scene - manages a collection of entities

use crate::components::*;
use crate::entity::{Entity, EntityId};
use crate::scene_data::{SerializedComponent, SerializedEntity, SerializedScene};
use crate::transform::Transform;
use glam::Mat4;
use std::collections::HashMap;

pub struct Scene {
    pub name: String,
    entities: HashMap<EntityId, Entity>,
    next_id: u64,
    root_entities: Vec<EntityId>,
}

impl Scene {
    pub fn new(name: String) -> Self {
        Self {
            name,
            entities: HashMap::new(),
            next_id: 1,
            root_entities: Vec::new(),
        }
    }

    /// Create a new entity in the scene
    pub fn create_entity(&mut self, name: String) -> EntityId {
        let id = EntityId::new(self.next_id);
        self.next_id += 1;

        let entity = Entity::new(id, name);
        self.entities.insert(id, entity);
        self.root_entities.push(id);

        id
    }

    /// Create an entity with a transform
    pub fn create_entity_with_transform(&mut self, name: String, transform: Transform) -> EntityId {
        let id = self.create_entity(name);
        if let Some(entity) = self.entities.get_mut(&id) {
            entity.transform = transform;
        }
        id
    }

    /// Get an entity by ID
    pub fn get_entity(&self, id: EntityId) -> Option<&Entity> {
        self.entities.get(&id)
    }

    /// Get a mutable entity by ID
    pub fn get_entity_mut(&mut self, id: EntityId) -> Option<&mut Entity> {
        self.entities.get_mut(&id)
    }

    /// Remove an entity from the scene
    pub fn remove_entity(&mut self, id: EntityId) -> bool {
        if let Some(entity) = self.entities.remove(&id) {
            // Remove from parent's children list
            if let Some(parent_id) = entity.parent {
                if let Some(parent) = self.entities.get_mut(&parent_id) {
                    parent.children.retain(|&child_id| child_id != id);
                }
            } else {
                // Remove from root entities
                self.root_entities.retain(|&root_id| root_id != id);
            }

            // Remove all children recursively
            let children = entity.children.clone();
            for child_id in children {
                self.remove_entity(child_id);
            }

            true
        } else {
            false
        }
    }

    /// Set the parent of an entity
    pub fn set_parent(&mut self, child_id: EntityId, parent_id: Option<EntityId>) {
        // Remove from current parent
        if let Some(child) = self.entities.get(&child_id) {
            if let Some(old_parent_id) = child.parent {
                if let Some(old_parent) = self.entities.get_mut(&old_parent_id) {
                    old_parent.children.retain(|&id| id != child_id);
                }
            } else {
                self.root_entities.retain(|&id| id != child_id);
            }
        }

        // Set new parent
        if let Some(child) = self.entities.get_mut(&child_id) {
            child.parent = parent_id;
        }

        // Add to new parent's children
        if let Some(parent_id) = parent_id {
            if let Some(parent) = self.entities.get_mut(&parent_id) {
                if !parent.children.contains(&child_id) {
                    parent.children.push(child_id);
                }
            }
        } else {
            // Add to root entities
            if !self.root_entities.contains(&child_id) {
                self.root_entities.push(child_id);
            }
        }
    }

    /// Get all entities
    pub fn entities(&self) -> impl Iterator<Item = &Entity> {
        self.entities.values()
    }

    /// Get all root entities (entities without parents)
    pub fn root_entities(&self) -> &[EntityId] {
        &self.root_entities
    }

    /// Calculate world matrix for an entity (including parent transforms)
    pub fn world_matrix(&self, entity_id: EntityId) -> Mat4 {
        if let Some(entity) = self.get_entity(entity_id) {
            if let Some(parent_id) = entity.parent {
                let parent_world = self.world_matrix(parent_id);
                entity.transform.world_matrix(parent_world)
            } else {
                entity.transform.matrix()
            }
        } else {
            Mat4::IDENTITY
        }
    }

    /// Get entity count
    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }

    /// Clear all entities
    pub fn clear(&mut self) {
        self.entities.clear();
        self.root_entities.clear();
        self.next_id = 1;
    }

    /// Duplicate an entity with all its components
    /// Returns the new entity ID, or None if the entity doesn't exist
    pub fn duplicate_entity(&mut self, entity_id: EntityId) -> Option<EntityId> {
        // Get the entity to clone (need to clone data before mutating)
        let entity = self.get_entity(entity_id)?;
        let mut new_entity = entity.clone();
        let parent_id = entity.parent;

        // Assign new ID
        let new_id = EntityId::new(self.next_id);
        self.next_id += 1;
        new_entity.id = new_id;

        // Rename to indicate copy
        new_entity.name = format!("{} (Copy)", new_entity.name);

        // Clear children (don't duplicate hierarchy by default)
        new_entity.children.clear();

        // Insert into scene
        self.entities.insert(new_id, new_entity);

        // Add to same parent or root
        if let Some(pid) = parent_id {
            if let Some(parent) = self.entities.get_mut(&pid) {
                parent.children.push(new_id);
            }
        } else {
            self.root_entities.push(new_id);
        }

        Some(new_id)
    }

    /// Convert scene to serializable format
    /// Note: Only core components (MeshRenderer, Camera, Light) are serialized.
    /// Physics components should be serialized separately using scene_with_extensions.
    pub fn to_serialized(&self) -> SerializedScene {
        let mut serialized_entities = HashMap::new();

        for (id, entity) in &self.entities {
            let mut components = Vec::new();

            // Check for each component type and serialize it
            if let Some(mesh_renderer) = entity.get_component::<MeshRenderer>() {
                components.push(SerializedComponent::MeshRenderer(mesh_renderer.clone()));
            }
            if let Some(camera) = entity.get_component::<Camera>() {
                components.push(SerializedComponent::Camera(camera.clone()));
            }
            if let Some(light) = entity.get_component::<Light>() {
                components.push(SerializedComponent::Light(light.clone()));
            }
            if let Some(particle_emitter) = entity.get_component::<ParticleEmitter>() {
                components.push(SerializedComponent::ParticleEmitter(particle_emitter.clone()));
            }
            if let Some(water) = entity.get_component::<Water>() {
                components.push(SerializedComponent::Water(water.clone()));
            }
            if let Some(terrain_water) = entity.get_component::<TerrainWater>() {
                components.push(SerializedComponent::TerrainWater(terrain_water.clone()));
            }
            if let Some(terrain_gen) = entity.get_component::<TerrainGenerator>() {
                components.push(SerializedComponent::TerrainGenerator(terrain_gen.clone()));
            }
            if let Some(foliage) = entity.get_component::<Foliage>() {
                components.push(SerializedComponent::Foliage(foliage.clone()));
            }

            serialized_entities.insert(
                *id,
                SerializedEntity {
                    id: entity.id,
                    name: entity.name.clone(),
                    transform: entity.transform,
                    parent: entity.parent,
                    children: entity.children.clone(),
                    components,
                },
            );
        }

        SerializedScene {
            name: self.name.clone(),
            entities: serialized_entities,
            next_id: self.next_id,
            root_entities: self.root_entities.clone(),
        }
    }

    /// Create scene from serialized format
    /// Note: Only core components are deserialized. Physics and other extension
    /// components should be deserialized using scene_with_extensions.
    pub fn from_serialized(data: SerializedScene) -> Self {
        let mut entities = HashMap::new();

        for (id, serialized_entity) in data.entities {
            let mut entity = Entity::new(serialized_entity.id, serialized_entity.name);
            entity.transform = serialized_entity.transform;
            entity.parent = serialized_entity.parent;
            entity.children = serialized_entity.children;

            // Add each serialized component
            for component in serialized_entity.components {
                match component {
                    SerializedComponent::MeshRenderer(c) => entity.add_component(c),
                    SerializedComponent::Camera(c) => entity.add_component(c),
                    SerializedComponent::Light(c) => entity.add_component(c),
                    SerializedComponent::ParticleEmitter(c) => entity.add_component(c),
                    SerializedComponent::Water(c) => entity.add_component(c),
                    SerializedComponent::TerrainWater(c) => entity.add_component(c),
                    SerializedComponent::TerrainGenerator(c) => entity.add_component(c),
                    SerializedComponent::Foliage(c) => entity.add_component(c),
                    SerializedComponent::Generic { .. } => {
                        // Generic components are not deserialized at this level
                        // They should be handled by extension systems
                    }
                }
            }

            entities.insert(id, entity);
        }

        Self {
            name: data.name,
            entities,
            next_id: data.next_id,
            root_entities: data.root_entities,
        }
    }

    /// Save scene to RON file
    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let serialized = self.to_serialized();
        let ron_string = ron::ser::to_string_pretty(&serialized, Default::default())?;
        std::fs::write(path, ron_string)?;
        Ok(())
    }

    /// Load scene from RON file
    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let ron_string = std::fs::read_to_string(path)?;
        let serialized: SerializedScene = ron::de::from_str(&ron_string)?;
        Ok(Self::from_serialized(serialized))
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new("Untitled Scene".to_string())
    }
}
