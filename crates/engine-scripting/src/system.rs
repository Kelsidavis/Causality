// Script system - manages script execution in the game loop

use crate::api;
use crate::components::Script;
use crate::runtime::ScriptRuntime;
use anyhow::Result;
use engine_scene::scene::Scene;
use glam::{Quat, Vec3};
use rhai::Map;

/// Script system - handles script initialization and update
pub struct ScriptSystem {
    runtime: ScriptRuntime,
}

impl ScriptSystem {
    pub fn new() -> Self {
        let mut runtime = ScriptRuntime::new();

        // Register API bindings
        api::register_api(runtime.engine_mut());

        Self { runtime }
    }

    /// Initialize scripts from scene entities
    pub fn initialize(&mut self, scene: &Scene) -> Result<()> {
        for entity in scene.entities() {
            if let Some(script) = entity.get_component::<Script>() {
                if script.enabled {
                    self.runtime.load_script(entity.id, script.source.clone())?;
                    log::info!("Loaded script for entity: {}", entity.name);
                }
            }
        }

        Ok(())
    }

    /// Call start() function on all scripts (called once after initialization)
    pub fn start(&mut self, scene: &mut Scene) -> Result<()> {
        let entity_ids: Vec<_> = scene.entities().map(|e| e.id).collect();

        for entity_id in entity_ids {
            if !self.runtime.has_script(entity_id) {
                continue;
            }

            // Create context for the script
            let entity = scene.get_entity(entity_id).unwrap();
            let mut context = Map::new();
            context.insert("entity_id".into(), rhai::Dynamic::from(entity_id.0 as i64));
            context.insert("position".into(), rhai::Dynamic::from(entity.transform.position));
            context.insert("rotation".into(), rhai::Dynamic::from(entity.transform.rotation));

            // Try to call start() function (optional)
            if let Err(e) = self.runtime.call_function(entity_id, "start", (context,)) {
                // It's OK if start() doesn't exist
                if !e.to_string().contains("Function not found") {
                    log::warn!("Error calling start() for entity {}: {}", entity.name, e);
                }
            }
        }

        Ok(())
    }

    /// Update all scripts
    pub fn update(&mut self, scene: &mut Scene, delta_time: f32) -> Result<()> {
        // Collect all entity IDs that have scripts
        let entity_ids: Vec<_> = scene
            .entities()
            .filter(|e| self.runtime.has_script(e.id))
            .map(|e| e.id)
            .collect();

        for entity_id in entity_ids {
            let entity = scene.get_entity(entity_id).unwrap();

            // Check if script is enabled
            if let Some(script) = entity.get_component::<Script>() {
                if !script.enabled {
                    continue;
                }
            }

            // Create context for the script
            let mut context = Map::new();
            context.insert("entity_id".into(), rhai::Dynamic::from(entity_id.0 as i64));
            context.insert("position".into(), rhai::Dynamic::from(entity.transform.position));
            context.insert("rotation".into(), rhai::Dynamic::from(entity.transform.rotation));
            context.insert("scale".into(), rhai::Dynamic::from(entity.transform.scale));
            context.insert("dt".into(), rhai::Dynamic::from(delta_time));

            // Call update() function
            match self.runtime.call_function(entity_id, "update", (context,)) {
                Ok(result) => {
                    // Check if script returned a modified context
                    if let Some(updated_context) = result.try_cast::<Map>() {
                        // Apply changes back to entity
                        if let Some(entity_mut) = scene.get_entity_mut(entity_id) {
                            if let Some(new_pos) = updated_context.get("position") {
                                if let Some(pos) = new_pos.clone().try_cast::<Vec3>() {
                                    entity_mut.transform.position = pos;
                                }
                            }
                            if let Some(new_rot) = updated_context.get("rotation") {
                                if let Some(rot) = new_rot.clone().try_cast::<Quat>() {
                                    entity_mut.transform.rotation = rot;
                                }
                            }
                            if let Some(new_scale) = updated_context.get("scale") {
                                if let Some(scale) = new_scale.clone().try_cast::<Vec3>() {
                                    entity_mut.transform.scale = scale;
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    if !e.to_string().contains("Function not found") {
                        log::error!(
                            "Script error in entity {}: {}",
                            scene.get_entity(entity_id).unwrap().name,
                            e
                        );
                    }
                }
            }
        }

        Ok(())
    }

    /// Reload a script
    pub fn reload_script(&mut self, entity_id: engine_scene::entity::EntityId, source: String) -> Result<()> {
        self.runtime.reload_script(entity_id, source)
    }

    /// Get runtime reference
    pub fn runtime(&self) -> &ScriptRuntime {
        &self.runtime
    }

    /// Get mutable runtime reference
    pub fn runtime_mut(&mut self) -> &mut ScriptRuntime {
        &mut self.runtime
    }

    /// Register audio API with script engine
    pub fn register_audio_api(&mut self, command_queue: crate::audio::AudioCommandQueue) {
        crate::audio::register_audio_api(self.runtime.engine_mut(), command_queue);
    }
}

impl Default for ScriptSystem {
    fn default() -> Self {
        Self::new()
    }
}
