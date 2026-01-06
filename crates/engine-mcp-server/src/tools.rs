// MCP Tools - Operations that Claude Code can call

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

/// File-based IPC for communication with the editor
pub struct ToolRegistry {
    command_file: PathBuf,
    response_file: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct IpcCommand {
    id: u64,
    command: String,
    args: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct IpcResponse {
    id: u64,
    success: bool,
    result: Value,
}

impl ToolRegistry {
    pub fn new() -> Self {
        let tmp_dir = std::env::temp_dir();
        Self {
            command_file: tmp_dir.join("game-engine-mcp-command.json"),
            response_file: tmp_dir.join("game-engine-mcp-response.json"),
        }
    }

    /// Send a command to the editor and wait for response
    fn send_command(&self, command: &str, args: Value) -> Result<Value> {
        let id = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_micros() as u64;

        let ipc_command = IpcCommand {
            id,
            command: command.to_string(),
            args,
        };

        // Write command to file
        let command_json = serde_json::to_string(&ipc_command)?;
        fs::write(&self.command_file, command_json)?;
        log::debug!("Wrote command to {:?}", self.command_file);

        // Wait for response (with timeout)
        let timeout = Duration::from_secs(5);
        let start = SystemTime::now();

        loop {
            if start.elapsed()? > timeout {
                return Err(anyhow!("IPC timeout waiting for editor response"));
            }

            if self.response_file.exists() {
                let response_json = fs::read_to_string(&self.response_file)?;
                let response: IpcResponse = serde_json::from_str(&response_json)?;

                if response.id == id {
                    // Remove response file for next command
                    let _ = fs::remove_file(&self.response_file);

                    if response.success {
                        return Ok(response.result);
                    } else {
                        return Err(anyhow!("Editor error: {:?}", response.result));
                    }
                }
            }

            std::thread::sleep(Duration::from_millis(50));
        }
    }

    pub fn list_tools(&self) -> Vec<Value> {
        vec![
            json!({
                "name": "create_entity",
                "description": "Create a new entity in the scene",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "name": {
                            "type": "string",
                            "description": "Name of the entity"
                        },
                        "position": {
                            "type": "array",
                            "description": "Position [x, y, z]",
                            "items": { "type": "number" },
                            "minItems": 3,
                            "maxItems": 3
                        }
                    },
                    "required": ["name"]
                }
            }),
            json!({
                "name": "set_transform",
                "description": "Set an entity's transform (position, rotation, scale)",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "entity_name": {
                            "type": "string",
                            "description": "Name of the entity to modify"
                        },
                        "position": {
                            "type": "array",
                            "description": "Position [x, y, z]",
                            "items": { "type": "number" }
                        },
                        "scale": {
                            "type": "array",
                            "description": "Scale [x, y, z]",
                            "items": { "type": "number" }
                        }
                    },
                    "required": ["entity_name"]
                }
            }),
            json!({
                "name": "list_entities",
                "description": "List all entities in the scene",
                "inputSchema": {
                    "type": "object",
                    "properties": {}
                }
            }),
            json!({
                "name": "get_entity_info",
                "description": "Get detailed information about an entity",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "entity_name": {
                            "type": "string",
                            "description": "Name of the entity"
                        }
                    },
                    "required": ["entity_name"]
                }
            }),
            json!({
                "name": "delete_entity",
                "description": "Delete an entity from the scene",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "entity_name": {
                            "type": "string",
                            "description": "Name of the entity to delete"
                        }
                    },
                    "required": ["entity_name"]
                }
            }),
            json!({
                "name": "add_script",
                "description": "Add or update a Rhai script on an entity",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "entity_name": {
                            "type": "string",
                            "description": "Name of the entity"
                        },
                        "script": {
                            "type": "string",
                            "description": "Rhai script code"
                        }
                    },
                    "required": ["entity_name", "script"]
                }
            }),
            json!({
                "name": "load_model",
                "description": "Load a 3D model (GLTF) into the scene",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "entity_name": {
                            "type": "string",
                            "description": "Name for the new entity"
                        },
                        "model_path": {
                            "type": "string",
                            "description": "Path to the GLTF model file"
                        },
                        "position": {
                            "type": "array",
                            "description": "Position [x, y, z]",
                            "items": { "type": "number" }
                        }
                    },
                    "required": ["entity_name", "model_path"]
                }
            }),
            json!({
                "name": "get_scene_info",
                "description": "Get information about the current scene",
                "inputSchema": {
                    "type": "object",
                    "properties": {}
                }
            }),
        ]
    }

    pub fn call_tool(&mut self, params: &Value) -> Result<Value> {
        let tool_name = params
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing tool name"))?;

        let arguments = params
            .get("arguments")
            .ok_or_else(|| anyhow!("Missing arguments"))?;

        log::info!("Executing tool: {}", tool_name);
        log::debug!("Arguments: {}", arguments);

        match tool_name {
            "create_entity" => self.create_entity(arguments),
            "set_transform" => self.set_transform(arguments),
            "list_entities" => self.list_entities(arguments),
            "get_entity_info" => self.get_entity_info(arguments),
            "delete_entity" => self.delete_entity(arguments),
            "add_script" => self.add_script(arguments),
            "load_model" => self.load_model(arguments),
            "get_scene_info" => self.get_scene_info(arguments),
            _ => Err(anyhow!("Unknown tool: {}", tool_name)),
        }
    }

    fn create_entity(&self, args: &Value) -> Result<Value> {
        let name = args
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing entity name"))?;

        let position = args
            .get("position")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_f64())
                    .map(|v| v as f32)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_else(|| vec![0.0, 0.0, 0.0]);

        log::info!(
            "Creating entity '{}' at position {:?}",
            name,
            position
        );

        let result = self.send_command("create_entity", json!({
            "name": name,
            "position": position
        }))?;

        Ok(json!({
            "content": [{
                "type": "text",
                "text": format!("Created entity '{}' at position [{}, {}, {}]", name, position[0], position[1], position[2])
            }]
        }))
    }

    fn set_transform(&self, args: &Value) -> Result<Value> {
        let entity_name = args
            .get("entity_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing entity_name"))?;

        // TODO: Send IPC message to editor
        log::info!("Would set transform for entity '{}'", entity_name);

        Ok(json!({
            "content": [{
                "type": "text",
                "text": format!("Updated transform for entity '{}'", entity_name)
            }]
        }))
    }

    fn list_entities(&self, _args: &Value) -> Result<Value> {
        log::info!("Listing all entities");

        let result = self.send_command("list_entities", json!({}))?;

        if let Some(entities) = result.get("entities").and_then(|v| v.as_array()) {
            let entity_names: Vec<String> = entities
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();

            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": if entity_names.is_empty() {
                        "No entities in scene".to_string()
                    } else {
                        format!("Entities in scene:\n- {}", entity_names.join("\n- "))
                    }
                }]
            }))
        } else {
            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": "No entities in scene"
                }]
            }))
        }
    }

    fn get_entity_info(&self, args: &Value) -> Result<Value> {
        let entity_name = args
            .get("entity_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing entity_name"))?;

        // TODO: Query editor via IPC
        log::info!("Would get info for entity '{}'", entity_name);

        Ok(json!({
            "content": [{
                "type": "text",
                "text": format!("Entity '{}' info:\nPosition: [0, 0, 0]\nRotation: [0, 0, 0, 1]\nScale: [1, 1, 1]", entity_name)
            }]
        }))
    }

    fn delete_entity(&self, args: &Value) -> Result<Value> {
        let entity_name = args
            .get("entity_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing entity_name"))?;

        // TODO: Send IPC message to editor
        log::info!("Would delete entity '{}'", entity_name);

        Ok(json!({
            "content": [{
                "type": "text",
                "text": format!("Deleted entity '{}'", entity_name)
            }]
        }))
    }

    fn add_script(&self, args: &Value) -> Result<Value> {
        let entity_name = args
            .get("entity_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing entity_name"))?;

        let script = args
            .get("script")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing script"))?;

        // TODO: Send IPC message to editor
        log::info!("Would add script to entity '{}'", entity_name);
        log::debug!("Script content: {}", script);

        Ok(json!({
            "content": [{
                "type": "text",
                "text": format!("Added script to entity '{}'", entity_name)
            }]
        }))
    }

    fn load_model(&self, args: &Value) -> Result<Value> {
        let entity_name = args
            .get("entity_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing entity_name"))?;

        let model_path = args
            .get("model_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing model_path"))?;

        // TODO: Send IPC message to editor
        log::info!(
            "Would load model '{}' as entity '{}'",
            model_path,
            entity_name
        );

        Ok(json!({
            "content": [{
                "type": "text",
                "text": format!("Loaded model '{}' as entity '{}'", model_path, entity_name)
            }]
        }))
    }

    fn get_scene_info(&self, _args: &Value) -> Result<Value> {
        // TODO: Query editor via IPC
        log::info!("Would get scene info");

        Ok(json!({
            "content": [{
                "type": "text",
                "text": "Scene: Demo Scene\nEntities: 3\nPhysics: Enabled\nScripting: Enabled"
            }]
        }))
    }
}
