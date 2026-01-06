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

        log::info!("Setting transform for entity '{}'", entity_name);

        let result = self.send_command("set_transform", json!({
            "entity_name": entity_name,
            "position": args.get("position"),
            "scale": args.get("scale"),
        }))?;

        let success = result.get("updated").and_then(|v| v.as_bool()).unwrap_or(false);
        let message = if success {
            format!("Updated transform for entity '{}'", entity_name)
        } else {
            result.get("error")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown error")
                .to_string()
        };

        Ok(json!({
            "content": [{
                "type": "text",
                "text": message
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

        log::info!("Getting info for entity '{}'", entity_name);

        let result = self.send_command("get_entity_info", json!({
            "entity_name": entity_name,
        }))?;

        if result.get("position").is_some() {
            let pos = result.get("position").and_then(|v| v.as_array()).cloned().unwrap_or_default();
            let rot = result.get("rotation").and_then(|v| v.as_array()).cloned().unwrap_or_default();
            let scl = result.get("scale").and_then(|v| v.as_array()).cloned().unwrap_or_default();

            let info = format!(
                "Entity: {}\nPosition: {:?}\nRotation: {:?}\nScale: {:?}",
                entity_name, pos, rot, scl
            );

            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": info
                }]
            }))
        } else {
            let error = result.get("error")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown error");

            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": format!("Error: {}", error)
                }]
            }))
        }
    }

    fn delete_entity(&self, args: &Value) -> Result<Value> {
        let entity_name = args
            .get("entity_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing entity_name"))?;

        log::info!("Deleting entity '{}'", entity_name);

        let result = self.send_command("delete_entity", json!({
            "name": entity_name,
        }))?;

        let success = result.get("deleted").and_then(|v| v.as_bool()).unwrap_or(false);
        let message = if success {
            format!("Deleted entity '{}'", entity_name)
        } else {
            result.get("error")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown error")
                .to_string()
        };

        Ok(json!({
            "content": [{
                "type": "text",
                "text": message
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

        log::info!("Adding script to entity '{}'", entity_name);
        log::debug!("Script content: {}", script);

        let result = self.send_command("add_script", json!({
            "entity_name": entity_name,
            "script": script,
        }))?;

        let error = result.get("error").and_then(|v| v.as_str());
        if error.is_some() {
            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": format!("Error: {}", error.unwrap())
                }]
            }))
        } else {
            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": format!("Added script to entity '{}'", entity_name)
                }]
            }))
        }
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

        log::info!(
            "Loading model '{}' as entity '{}'",
            model_path,
            entity_name
        );

        let result = self.send_command("load_model", json!({
            "entity_name": entity_name,
            "model_path": model_path,
        }))?;

        let error = result.get("error").and_then(|v| v.as_str());
        if error.is_some() {
            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": format!("Error: {}", error.unwrap())
                }]
            }))
        } else {
            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": format!("Loaded model '{}' as entity '{}'", model_path, entity_name)
                }]
            }))
        }
    }

    fn get_scene_info(&self, _args: &Value) -> Result<Value> {
        log::info!("Getting scene info");

        let result = self.send_command("get_scene_info", json!({}))?;

        if let Some(entity_count) = result.get("entity_count").and_then(|v| v.as_u64()) {
            let entities: Vec<String> = result
                .get("entities")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();

            let entity_list = if entities.is_empty() {
                "No entities".to_string()
            } else {
                entities.join(", ")
            };

            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": format!("Scene Info:\nEntities: {}\n- {}", entity_count, entity_list)
                }]
            }))
        } else {
            Ok(json!({
                "content": [{
                    "type": "text",
                    "text": "Scene: Unknown\nEntities: 0"
                }]
            }))
        }
    }
}
