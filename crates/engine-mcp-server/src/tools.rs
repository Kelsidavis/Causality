// MCP Tools - Operations that Claude Code can call

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;
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
            json!({
                "name": "add_rigidbody",
                "description": "Add a physics rigid body component to an entity",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "entity_name": {
                            "type": "string",
                            "description": "Name of the entity"
                        },
                        "body_type": {
                            "type": "string",
                            "description": "Body type: 'dynamic', 'kinematic', or 'static' (default: dynamic)"
                        },
                        "mass": {
                            "type": "number",
                            "description": "Mass for dynamic bodies (default: 1.0)"
                        }
                    },
                    "required": ["entity_name"]
                }
            }),
            json!({
                "name": "add_collider",
                "description": "Add a physics collider component to an entity",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "entity_name": {
                            "type": "string",
                            "description": "Name of the entity"
                        },
                        "shape_type": {
                            "type": "string",
                            "description": "Shape: 'box', 'sphere', or 'capsule' (default: box)"
                        },
                        "size": {
                            "type": "array",
                            "description": "Size [x, y, z] for box collider",
                            "items": { "type": "number" }
                        },
                        "radius": {
                            "type": "number",
                            "description": "Radius for sphere and capsule colliders (default: 0.5)"
                        },
                        "height": {
                            "type": "number",
                            "description": "Height for capsule collider (default: 1.0)"
                        }
                    },
                    "required": ["entity_name"]
                }
            }),
            json!({
                "name": "generate_texture",
                "description": "Generate a texture from a text prompt using AI (Stable Diffusion)",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "prompt": {
                            "type": "string",
                            "description": "Text description of the texture to generate"
                        },
                        "width": {
                            "type": "integer",
                            "description": "Texture width in pixels (default: 512, must be multiple of 64)"
                        },
                        "height": {
                            "type": "integer",
                            "description": "Texture height in pixels (default: 512, must be multiple of 64)"
                        },
                        "quality": {
                            "type": "string",
                            "description": "Quality level: 'fast', 'standard', 'high', or 'best' (default: 'high')"
                        },
                        "seed": {
                            "type": "integer",
                            "description": "Random seed for reproducibility (optional)"
                        }
                    },
                    "required": ["prompt"]
                }
            }),
            json!({
                "name": "generate_skybox",
                "description": "Generate a 360-degree skybox from a text prompt using AI",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "prompt": {
                            "type": "string",
                            "description": "Text description of the skybox environment"
                        },
                        "quality": {
                            "type": "string",
                            "description": "Quality level: 'fast', 'standard', 'high', or 'best' (default: 'high')"
                        },
                        "seed": {
                            "type": "integer",
                            "description": "Random seed for reproducibility (optional)"
                        }
                    },
                    "required": ["prompt"]
                }
            }),
            json!({
                "name": "save_scene",
                "description": "Save the current scene to a file",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "file_path": {
                            "type": "string",
                            "description": "Path to save the scene file (e.g., 'assets/scenes/my_scene.ron')"
                        }
                    },
                    "required": ["file_path"]
                }
            }),
            json!({
                "name": "load_scene",
                "description": "Load a scene from a file",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "file_path": {
                            "type": "string",
                            "description": "Path to the scene file to load (e.g., 'assets/scenes/castle.ron')"
                        }
                    },
                    "required": ["file_path"]
                }
            }),
            json!({
                "name": "generate_music",
                "description": "Generate music from a text description using AI. Claude should use external tools (like ACE-Step) to generate the actual audio, then return it to the game engine.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "prompt": {
                            "type": "string",
                            "description": "Text description of the music to generate (e.g., 'Epic orchestral battle music with drums')"
                        },
                        "duration": {
                            "type": "string",
                            "description": "Duration: 'short' (~15s), 'medium' (~30s), 'long' (~60s), or 'extended' (~2min). Default: 'medium'"
                        },
                        "style": {
                            "type": "string",
                            "description": "Music style/genre: 'rock', 'pop', 'electronic', 'jazz', 'classical', 'cinematic', 'ambient', etc. Default: based on prompt"
                        },
                        "tempo": {
                            "type": "integer",
                            "description": "Tempo in BPM (e.g., 120). Optional."
                        },
                        "instrumental": {
                            "type": "boolean",
                            "description": "True for instrumental only (no vocals). Default: true"
                        },
                        "output_path": {
                            "type": "string",
                            "description": "Where to save the generated music file (e.g., 'assets/music/battle.wav')"
                        }
                    },
                    "required": ["prompt", "output_path"]
                }
            }),
            json!({
                "name": "play_music",
                "description": "Play a music file in the game engine",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "file_path": {
                            "type": "string",
                            "description": "Path to the music file to play"
                        },
                        "loop": {
                            "type": "boolean",
                            "description": "Whether to loop the music. Default: true"
                        },
                        "volume": {
                            "type": "number",
                            "description": "Volume level (0.0 to 1.0). Default: 0.8"
                        },
                        "fade_in": {
                            "type": "number",
                            "description": "Fade in duration in seconds. Default: 0.0 (no fade)"
                        }
                    },
                    "required": ["file_path"]
                }
            }),
            json!({
                "name": "stop_music",
                "description": "Stop currently playing music",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "fade_out": {
                            "type": "number",
                            "description": "Fade out duration in seconds. Default: 0.0 (stop immediately)"
                        }
                    }
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
            "add_rigidbody" => self.add_rigidbody(arguments),
            "add_collider" => self.add_collider(arguments),
            "generate_texture" => self.generate_texture(arguments),
            "generate_skybox" => self.generate_skybox(arguments),
            "save_scene" => self.save_scene(arguments),
            "load_scene" => self.load_scene(arguments),
            "generate_music" => self.generate_music(arguments),
            "play_music" => self.play_music(arguments),
            "stop_music" => self.stop_music(arguments),
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
        log::debug!("Script length: {} bytes", script.len());

        let result = self.send_command("add_script", json!({
            "entity_name": entity_name,
            "script": script,
        }))?;

        let success = result.get("script_added").and_then(|v| v.as_bool()).unwrap_or(false);
        let message = if success {
            let size = result.get("script_size").and_then(|v| v.as_u64()).unwrap_or(0);
            format!("Successfully added script ({} bytes) to entity '{}'", size, entity_name)
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

    fn load_model(&self, args: &Value) -> Result<Value> {
        let entity_name = args
            .get("entity_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing entity_name"))?;

        let model_path = args
            .get("model_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing model_path"))?;

        let position = args
            .get("position")
            .and_then(|v| v.as_array())
            .cloned();

        log::info!(
            "Loading model '{}' as entity '{}'",
            model_path,
            entity_name
        );

        let mut load_args = json!({
            "entity_name": entity_name,
            "model_path": model_path,
        });

        if let Some(pos) = position {
            load_args["position"] = serde_json::Value::Array(pos);
        }

        let result = self.send_command("load_model", load_args)?;

        let success = result.get("model_loaded").and_then(|v| v.as_bool()).unwrap_or(false);
        let message = if success {
            let pos = result.get("position").and_then(|v| v.as_array()).cloned().unwrap_or_default();
            format!("Successfully loaded model '{}' as entity '{}' at position {:?}", model_path, entity_name, pos)
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

    fn add_rigidbody(&self, args: &Value) -> Result<Value> {
        let entity_name = args
            .get("entity_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing entity_name"))?;

        let body_type = args
            .get("body_type")
            .and_then(|v| v.as_str())
            .unwrap_or("dynamic");

        let mass = args
            .get("mass")
            .and_then(|v| v.as_f64())
            .unwrap_or(1.0);

        log::info!("Adding {} rigidbody to entity '{}'", body_type, entity_name);

        let result = self.send_command("add_rigidbody", json!({
            "entity_name": entity_name,
            "body_type": body_type,
            "mass": mass,
        }))?;

        let success = result.get("rigidbody_added").and_then(|v| v.as_bool()).unwrap_or(false);
        let message = if success {
            format!("Successfully added {} rigidbody to entity '{}'", body_type, entity_name)
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

    fn add_collider(&self, args: &Value) -> Result<Value> {
        let entity_name = args
            .get("entity_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing entity_name"))?;

        let shape_type = args
            .get("shape_type")
            .and_then(|v| v.as_str())
            .unwrap_or("box");

        let size = args.get("size").and_then(|v| v.as_array()).cloned();
        let radius = args.get("radius").and_then(|v| v.as_f64()).unwrap_or(0.5);
        let height = args.get("height").and_then(|v| v.as_f64());

        log::info!("Adding {} collider to entity '{}'", shape_type, entity_name);

        let mut collider_args = json!({
            "entity_name": entity_name,
            "shape_type": shape_type,
            "radius": radius,
        });

        if let Some(sz) = size {
            collider_args["size"] = serde_json::Value::Array(sz);
        }

        if let Some(h) = height {
            collider_args["height"] = json!(h);
        }

        let result = self.send_command("add_collider", collider_args)?;

        let success = result.get("collider_added").and_then(|v| v.as_bool()).unwrap_or(false);
        let message = if success {
            format!("Successfully added {} collider to entity '{}'", shape_type, entity_name)
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

    fn generate_texture(&self, args: &Value) -> Result<Value> {
        let prompt = args
            .get("prompt")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing prompt"))?;

        let width = args
            .get("width")
            .and_then(|v| v.as_u64())
            .unwrap_or(512) as u32;

        let height = args
            .get("height")
            .and_then(|v| v.as_u64())
            .unwrap_or(512) as u32;

        let quality = args
            .get("quality")
            .and_then(|v| v.as_str())
            .unwrap_or("high");

        let seed = args.get("seed").and_then(|v| v.as_u64());

        log::info!(
            "Generating texture '{}' ({}x{}, quality: {})",
            prompt,
            width,
            height,
            quality
        );

        let result = self.send_command("generate_texture", json!({
            "prompt": prompt,
            "width": width,
            "height": height,
            "quality": quality,
            "seed": seed,
        }))?;

        let success = result.get("generated").and_then(|v| v.as_bool()).unwrap_or(false);
        let asset_id = result.get("asset_id").and_then(|v| v.as_str());

        let message = if success {
            if let Some(id) = asset_id {
                format!("Successfully generated texture '{}' (ID: {})", prompt, id)
            } else {
                format!("Successfully generated texture '{}'", prompt)
            }
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

    fn generate_skybox(&self, args: &Value) -> Result<Value> {
        let prompt = args
            .get("prompt")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing prompt"))?;

        let quality = args
            .get("quality")
            .and_then(|v| v.as_str())
            .unwrap_or("high");

        let seed = args.get("seed").and_then(|v| v.as_u64());

        log::info!("Generating skybox '{}' (quality: {})", prompt, quality);

        let result = self.send_command("generate_skybox", json!({
            "prompt": prompt,
            "quality": quality,
            "seed": seed,
        }))?;

        let success = result.get("generated").and_then(|v| v.as_bool()).unwrap_or(false);
        let asset_id = result.get("asset_id").and_then(|v| v.as_str());

        let message = if success {
            if let Some(id) = asset_id {
                format!("Successfully generated skybox '{}' (ID: {})", prompt, id)
            } else {
                format!("Successfully generated skybox '{}'", prompt)
            }
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

    fn save_scene(&self, args: &Value) -> Result<Value> {
        let file_path = args
            .get("file_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing file_path"))?;

        log::info!("Saving scene to '{}'", file_path);

        let result = self.send_command("save_scene", json!({
            "file_path": file_path,
        }))?;

        let success = result.get("saved").and_then(|v| v.as_bool()).unwrap_or(false);
        let message = if success {
            let entity_count = result.get("entity_count").and_then(|v| v.as_u64()).unwrap_or(0);
            format!("Successfully saved scene to '{}' ({} entities)", file_path, entity_count)
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

    fn load_scene(&self, args: &Value) -> Result<Value> {
        let file_path = args
            .get("file_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing file_path"))?;

        log::info!("Loading scene from '{}'", file_path);

        let result = self.send_command("load_scene", json!({
            "file_path": file_path,
        }))?;

        let success = result.get("loaded").and_then(|v| v.as_bool()).unwrap_or(false);
        let message = if success {
            let entity_count = result.get("entity_count").and_then(|v| v.as_u64()).unwrap_or(0);
            let scene_name = result.get("scene_name").and_then(|v| v.as_str()).unwrap_or("Unknown");
            format!("Successfully loaded scene '{}' from '{}' ({} entities)", scene_name, file_path, entity_count)
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

    fn generate_music(&self, args: &Value) -> Result<Value> {
        let prompt = args
            .get("prompt")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing prompt"))?;

        let output_path = args
            .get("output_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing output_path"))?;

        let duration = args
            .get("duration")
            .and_then(|v| v.as_str())
            .unwrap_or("medium");

        let style = args.get("style").and_then(|v| v.as_str());
        let tempo = args.get("tempo").and_then(|v| v.as_u64());
        let instrumental = args.get("instrumental").and_then(|v| v.as_bool()).unwrap_or(true);

        log::info!(
            "Music generation requested: '{}' -> '{}'",
            prompt,
            output_path
        );

        // Return a message for Claude to handle
        // Claude will see this and know to use WebFetch or other tools to call ACE-Step
        Ok(json!({
            "content": [{
                "type": "text",
                "text": format!(
                    "Music generation request received:\n\
                     Prompt: {}\n\
                     Duration: {}\n\
                     Style: {}\n\
                     Tempo: {}\n\
                     Instrumental: {}\n\
                     Output: {}\n\n\
                     Please use the ACE-Step API at http://localhost:7865 to generate this music.\n\
                     Then save the generated audio to '{}' and confirm completion.",
                    prompt,
                    duration,
                    style.unwrap_or("auto"),
                    tempo.map(|t| t.to_string()).unwrap_or_else(|| "auto".to_string()),
                    instrumental,
                    output_path,
                    output_path
                )
            }]
        }))
    }

    fn play_music(&self, args: &Value) -> Result<Value> {
        let file_path = args
            .get("file_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing file_path"))?;

        let loop_music = args.get("loop").and_then(|v| v.as_bool()).unwrap_or(true);
        let volume = args.get("volume").and_then(|v| v.as_f64()).unwrap_or(0.8);
        let fade_in = args.get("fade_in").and_then(|v| v.as_f64()).unwrap_or(0.0);

        log::info!("Playing music: '{}' (loop: {}, volume: {})", file_path, loop_music, volume);

        let result = self.send_command("play_music", json!({
            "file_path": file_path,
            "loop": loop_music,
            "volume": volume,
            "fade_in": fade_in,
        }))?;

        let success = result.get("playing").and_then(|v| v.as_bool()).unwrap_or(false);
        let message = if success {
            format!("Now playing: '{}' (volume: {}, loop: {})", file_path, volume, loop_music)
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

    fn stop_music(&self, args: &Value) -> Result<Value> {
        let fade_out = args.get("fade_out").and_then(|v| v.as_f64()).unwrap_or(0.0);

        log::info!("Stopping music (fade_out: {}s)", fade_out);

        let result = self.send_command("stop_music", json!({
            "fade_out": fade_out,
        }))?;

        let success = result.get("stopped").and_then(|v| v.as_bool()).unwrap_or(false);
        let message = if success {
            if fade_out > 0.0 {
                format!("Music stopping (fading out over {}s)", fade_out)
            } else {
                "Music stopped".to_string()
            }
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
}
