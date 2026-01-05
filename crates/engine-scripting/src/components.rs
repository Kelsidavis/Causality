// Script component for entities

use engine_scene::entity::Component;
use engine_scene::impl_component;
use serde::{Deserialize, Serialize};
use std::any::Any;

/// Script component - attaches a Rhai script to an entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Script {
    pub source: String,
    pub enabled: bool,
}

impl Script {
    pub fn new(source: String) -> Self {
        Self {
            source,
            enabled: true,
        }
    }

    pub fn from_file(path: &str) -> std::io::Result<Self> {
        let source = std::fs::read_to_string(path)?;
        Ok(Self::new(source))
    }
}

impl_component!(Script);
