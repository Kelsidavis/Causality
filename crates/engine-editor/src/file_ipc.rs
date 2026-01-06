// File-based IPC handler for MCP server communication

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;
use engine_scene::Scene;
use engine_scene::components::MeshRenderer;
use engine_scripting::Script;
use glam::Vec3;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcCommand {
    pub id: u64,
    pub command: String,
    pub args: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcResponse {
    pub id: u64,
    pub success: bool,
    pub result: Value,
}

pub struct FileIpcHandler {
    command_file: PathBuf,
    response_file: PathBuf,
}

impl FileIpcHandler {
    pub fn new() -> Self {
        let tmp_dir = std::env::temp_dir();
        Self {
            command_file: tmp_dir.join("game-engine-mcp-command.json"),
            response_file: tmp_dir.join("game-engine-mcp-response.json"),
        }
    }

    /// Check for incoming commands (non-blocking)
    pub fn poll_commands(&mut self, scene: &mut Scene) -> Result<()> {
        if !self.command_file.exists() {
            return Ok(());
        }

        // Read command
        let command_json = match fs::read_to_string(&self.command_file) {
            Ok(json) => json,
            Err(_) => return Ok(()), // File might be being written
        };

        let command: IpcCommand = match serde_json::from_str(&command_json) {
            Ok(cmd) => cmd,
            Err(e) => {
                log::error!("Failed to parse IPC command: {}", e);
                return Ok(());
            }
        };

        // Remove command file
        let _ = fs::remove_file(&self.command_file);

        log::info!("Processing IPC command: {}", command.command);

        // Execute command
        let response = self.execute_command(command.id, &command.command, command.args, scene);

        // Write response
        let response_json = serde_json::to_string(&response)?;
        fs::write(&self.response_file, response_json)?;

        Ok(())
    }

    fn execute_command(
        &self,
        id: u64,
        command: &str,
        args: Value,
        scene: &mut Scene,
    ) -> IpcResponse {
        match command {
            "create_entity" => {
                let name = args
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("New Entity");

                let position = args
                    .get("position")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        glam::Vec3::new(
                            arr.get(0).and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
                            arr.get(1).and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
                            arr.get(2).and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
                        )
                    })
                    .unwrap_or(glam::Vec3::ZERO);

                let entity_id = scene.create_entity(name.to_string());
                if let Some(entity) = scene.get_entity_mut(entity_id) {
                    entity.transform.position = position;
                }

                log::info!("Created entity '{}' at {:?}", name, position);

                IpcResponse {
                    id,
                    success: true,
                    result: json!({
                        "entity_id": format!("{:?}", entity_id),
                        "name": name
                    }),
                }
            }
            "list_entities" => {
                let entities: Vec<String> = scene
                    .entities()
                    .map(|e| e.name.clone())
                    .collect();

                IpcResponse {
                    id,
                    success: true,
                    result: json!({
                        "entities": entities,
                        "count": entities.len()
                    }),
                }
            }
            "get_scene_info" => {
                let entity_count = scene.entities().count();
                let entities: Vec<String> = scene
                    .entities()
                    .map(|e| e.name.clone())
                    .collect();

                IpcResponse {
                    id,
                    success: true,
                    result: json!({
                        "entity_count": entity_count,
                        "entities": entities
                    }),
                }
            }
            "delete_entity" => {
                let name = args
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                // Find entity by name
                let entity_id = scene
                    .entities()
                    .find(|e| e.name == name)
                    .map(|e| e.id);

                if let Some(id) = entity_id {
                    scene.remove_entity(id);
                    log::info!("Deleted entity '{}'", name);

                    IpcResponse {
                        id: id.0,
                        success: true,
                        result: json!({
                            "deleted": true,
                            "name": name
                        }),
                    }
                } else {
                    IpcResponse {
                        id: id.try_into().unwrap_or(0),
                        success: false,
                        result: json!({
                            "error": format!("Entity '{}' not found", name)
                        }),
                    }
                }
            }
            "set_transform" => {
                let entity_name = args
                    .get("entity_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                // Find entity by name
                let entity_id_opt = scene
                    .entities()
                    .find(|e| e.name == entity_name)
                    .map(|e| e.id);

                if let Some(entity_id) = entity_id_opt {
                    if let Some(entity) = scene.get_entity_mut(entity_id) {
                        // Update position if provided
                        if let Some(pos) = args.get("position").and_then(|v| v.as_array()) {
                            entity.transform.position = glam::Vec3::new(
                                pos.get(0).and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
                                pos.get(1).and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
                                pos.get(2).and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
                            );
                        }

                        // Update scale if provided
                        if let Some(scl) = args.get("scale").and_then(|v| v.as_array()) {
                            entity.transform.scale = glam::Vec3::new(
                                scl.get(0).and_then(|v| v.as_f64()).unwrap_or(1.0) as f32,
                                scl.get(1).and_then(|v| v.as_f64()).unwrap_or(1.0) as f32,
                                scl.get(2).and_then(|v| v.as_f64()).unwrap_or(1.0) as f32,
                            );
                        }

                        log::info!("Updated transform for entity '{}'", entity_name);

                        IpcResponse {
                            id,
                            success: true,
                            result: json!({
                                "updated": true,
                                "entity_name": entity_name
                            }),
                        }
                    } else {
                        IpcResponse {
                            id,
                            success: false,
                            result: json!({
                                "error": format!("Failed to get entity '{}'", entity_name)
                            }),
                        }
                    }
                } else {
                    IpcResponse {
                        id,
                        success: false,
                        result: json!({
                            "error": format!("Entity '{}' not found", entity_name)
                        }),
                    }
                }
            }
            "get_entity_info" => {
                let entity_name = args
                    .get("entity_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                let entity_id_opt = scene
                    .entities()
                    .find(|e| e.name == entity_name)
                    .map(|e| e.id);

                if let Some(entity_id) = entity_id_opt {
                    if let Some(entity) = scene.get_entity(entity_id) {
                        IpcResponse {
                            id,
                            success: true,
                            result: json!({
                                "entity_id": format!("{:?}", entity_id),
                                "name": entity.name,
                                "position": [entity.transform.position.x, entity.transform.position.y, entity.transform.position.z],
                                "rotation": [entity.transform.rotation.x, entity.transform.rotation.y, entity.transform.rotation.z, entity.transform.rotation.w],
                                "scale": [entity.transform.scale.x, entity.transform.scale.y, entity.transform.scale.z],
                            }),
                        }
                    } else {
                        IpcResponse {
                            id,
                            success: false,
                            result: json!({
                                "error": format!("Failed to get entity '{}'", entity_name)
                            }),
                        }
                    }
                } else {
                    IpcResponse {
                        id,
                        success: false,
                        result: json!({
                            "error": format!("Entity '{}' not found", entity_name)
                        }),
                    }
                }
            }
            "add_script" => {
                let entity_name = args
                    .get("entity_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                let script_source = args
                    .get("script")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                // Find entity by name
                let entity_id_opt = scene
                    .entities()
                    .find(|e| e.name == entity_name)
                    .map(|e| e.id);

                if let Some(entity_id) = entity_id_opt {
                    if let Some(entity) = scene.get_entity_mut(entity_id) {
                        // Create and attach script component
                        let script = Script::new(script_source.to_string());
                        entity.add_component(script);

                        log::info!("Added script to entity '{}' (length: {} bytes)", entity_name, script_source.len());

                        IpcResponse {
                            id,
                            success: true,
                            result: json!({
                                "script_added": true,
                                "entity_name": entity_name,
                                "script_size": script_source.len()
                            }),
                        }
                    } else {
                        IpcResponse {
                            id,
                            success: false,
                            result: json!({
                                "error": format!("Failed to get entity '{}'", entity_name)
                            }),
                        }
                    }
                } else {
                    IpcResponse {
                        id,
                        success: false,
                        result: json!({
                            "error": format!("Entity '{}' not found", entity_name)
                        }),
                    }
                }
            }
            "load_model" => {
                let entity_name = args
                    .get("entity_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                let model_path = args
                    .get("model_path")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                let position = args
                    .get("position")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        Vec3::new(
                            arr.get(0).and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
                            arr.get(1).and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
                            arr.get(2).and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
                        )
                    })
                    .unwrap_or(Vec3::ZERO);

                // Create new entity with model
                let entity_id = scene.create_entity(entity_name.to_string());

                if let Some(entity) = scene.get_entity_mut(entity_id) {
                    // Set position
                    entity.transform.position = position;

                    // Add mesh renderer component
                    let mesh_renderer = MeshRenderer {
                        mesh_path: model_path.to_string(),
                        material_path: None,
                    };
                    entity.add_component(mesh_renderer);

                    log::info!("Loaded model '{}' as entity '{}' at position {:?}",
                        model_path, entity_name, position);

                    IpcResponse {
                        id,
                        success: true,
                        result: json!({
                            "model_loaded": true,
                            "entity_name": entity_name,
                            "model_path": model_path,
                            "position": [position.x, position.y, position.z]
                        }),
                    }
                } else {
                    IpcResponse {
                        id,
                        success: false,
                        result: json!({
                            "error": format!("Failed to create entity '{}'", entity_name)
                        }),
                    }
                }
            }
            _ => IpcResponse {
                id,
                success: false,
                result: json!({
                    "error": format!("Unknown command: {}", command)
                }),
            },
        }
    }
}
