// Entity and Component system

use crate::transform::Transform;
use serde::{Deserialize, Serialize};
use std::any::Any;

/// Unique identifier for entities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityId(pub u64);

impl EntityId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

/// Component trait - all components must implement this
pub trait Component: Any {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn clone_box(&self) -> Box<dyn Component>;
}

/// Helper macro to implement Component trait
#[macro_export]
macro_rules! impl_component {
    ($type:ty) => {
        impl Component for $type {
            fn as_any(&self) -> &dyn Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut dyn Any {
                self
            }

            fn clone_box(&self) -> Box<dyn Component> {
                Box::new(self.clone())
            }
        }
    };
}

/// Entity - represents an object in the scene
pub struct Entity {
    pub id: EntityId,
    pub name: String,
    pub transform: Transform,
    pub parent: Option<EntityId>,
    pub children: Vec<EntityId>,
    components: Vec<Box<dyn Component>>,
}

impl Entity {
    pub fn new(id: EntityId, name: String) -> Self {
        Self {
            id,
            name,
            transform: Transform::default(),
            parent: None,
            children: Vec::new(),
            components: Vec::new(),
        }
    }

    pub fn with_transform(mut self, transform: Transform) -> Self {
        self.transform = transform;
        self
    }

    /// Add a component to this entity
    pub fn add_component<T: Component + 'static>(&mut self, component: T) {
        self.components.push(Box::new(component));
    }

    /// Get a component by type
    pub fn get_component<T: Component + 'static>(&self) -> Option<&T> {
        for component in &self.components {
            if let Some(c) = component.as_any().downcast_ref::<T>() {
                return Some(c);
            }
        }
        None
    }

    /// Get a mutable component by type
    pub fn get_component_mut<T: Component + 'static>(&mut self) -> Option<&mut T> {
        for component in &mut self.components {
            if let Some(c) = component.as_any_mut().downcast_mut::<T>() {
                return Some(c);
            }
        }
        None
    }

    /// Remove a component by type
    pub fn remove_component<T: Component + 'static>(&mut self) -> bool {
        let initial_len = self.components.len();
        self.components.retain(|c| c.as_any().downcast_ref::<T>().is_none());
        self.components.len() < initial_len
    }

    /// Check if entity has a component of type T
    pub fn has_component<T: Component + 'static>(&self) -> bool {
        self.get_component::<T>().is_some()
    }

    /// Get all components
    pub fn components(&self) -> &[Box<dyn Component>] {
        &self.components
    }
}

impl Clone for Entity {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            name: self.name.clone(),
            transform: self.transform,
            parent: self.parent,
            children: self.children.clone(),
            components: self.components.iter().map(|c| c.clone_box()).collect(),
        }
    }
}
