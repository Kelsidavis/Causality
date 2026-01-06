// File-based IPC handler for MCP server communication

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;
use engine_scene::Scene;

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
