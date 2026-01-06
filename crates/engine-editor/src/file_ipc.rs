// File-based IPC handler for MCP server communication

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;
use engine_scene::Scene;
use engine_scene::components::MeshRenderer;
use engine_scripting::Script;
use engine_physics::{RigidBody, RigidBodyType, Collider, ColliderShape};
use engine_ai_assets::{AssetGenerator, AssetCache, TextureGenerationRequest, LocalClient, AiAssetConfig};
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
            "add_rigidbody" => {
                let entity_name = args
                    .get("entity_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                let body_type_str = args
                    .get("body_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("dynamic");

                let mass = args
                    .get("mass")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(1.0) as f32;

                // Find entity by name
                let entity_id_opt = scene
                    .entities()
                    .find(|e| e.name == entity_name)
                    .map(|e| e.id);

                if let Some(entity_id) = entity_id_opt {
                    if let Some(entity) = scene.get_entity_mut(entity_id) {
                        // Create rigid body based on type
                        let rigidbody = match body_type_str {
                            "static" => RigidBody::static_body(),
                            "kinematic" => RigidBody::kinematic(),
                            "dynamic" | _ => RigidBody::dynamic(mass),
                        };

                        entity.add_component(rigidbody);

                        log::info!("Added {} rigidbody to entity '{}'", body_type_str, entity_name);

                        IpcResponse {
                            id,
                            success: true,
                            result: json!({
                                "rigidbody_added": true,
                                "entity_name": entity_name,
                                "body_type": body_type_str,
                                "mass": mass
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
            "add_collider" => {
                let entity_name = args
                    .get("entity_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                let shape_type = args
                    .get("shape_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("box");

                // Find entity by name
                let entity_id_opt = scene
                    .entities()
                    .find(|e| e.name == entity_name)
                    .map(|e| e.id);

                if let Some(entity_id) = entity_id_opt {
                    if let Some(entity) = scene.get_entity_mut(entity_id) {
                        // Parse size/radius parameters
                        let size = args.get("size").and_then(|v| v.as_array()).map(|arr| {
                            Vec3::new(
                                arr.get(0).and_then(|v| v.as_f64()).unwrap_or(0.5) as f32,
                                arr.get(1).and_then(|v| v.as_f64()).unwrap_or(0.5) as f32,
                                arr.get(2).and_then(|v| v.as_f64()).unwrap_or(0.5) as f32,
                            )
                        }).unwrap_or(Vec3::splat(0.5));

                        let radius = args.get("radius").and_then(|v| v.as_f64()).unwrap_or(0.5) as f32;

                        // Create collider based on type
                        let collider = match shape_type {
                            "sphere" => Collider::sphere(radius),
                            "capsule" => {
                                let height = args.get("height").and_then(|v| v.as_f64()).unwrap_or(1.0) as f32;
                                Collider::capsule(height / 2.0, radius)
                            },
                            "box" | _ => Collider::box_collider(size),
                        };

                        entity.add_component(collider);

                        log::info!("Added {} collider to entity '{}'", shape_type, entity_name);

                        IpcResponse {
                            id,
                            success: true,
                            result: json!({
                                "collider_added": true,
                                "entity_name": entity_name,
                                "shape_type": shape_type,
                                "size": [size.x, size.y, size.z],
                                "radius": radius
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
            "generate_texture" => {
                let prompt = args
                    .get("prompt")
                    .and_then(|v| v.as_str())
                    .unwrap_or("texture")
                    .to_string();

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
                    .unwrap_or("high")
                    .to_string();

                let seed = args.get("seed").and_then(|v| v.as_u64());

                log::info!(
                    "Generating texture '{}' ({}x{}, quality: {})",
                    prompt,
                    width,
                    height,
                    quality
                );

                // Clone for use after spawn
                let prompt_for_response = prompt.clone();

                // Try to generate texture using local Stable Diffusion service
                match std::thread::spawn(move || {
                    generate_texture_blocking(&prompt, width, height, &quality, seed)
                }).join() {
                    Ok(Ok((asset_id, file_path))) => {
                        IpcResponse {
                            id,
                            success: true,
                            result: json!({
                                "generated": true,
                                "asset_id": asset_id,
                                "file_path": file_path,
                                "prompt": prompt_for_response,
                                "dimensions": [width, height]
                            }),
                        }
                    }
                    Ok(Err(e)) => {
                        log::error!("Texture generation failed: {}", e);
                        IpcResponse {
                            id,
                            success: false,
                            result: json!({
                                "generated": false,
                                "error": format!("Texture generation failed: {}", e)
                            }),
                        }
                    }
                    Err(_) => {
                        IpcResponse {
                            id,
                            success: false,
                            result: json!({
                                "generated": false,
                                "error": "Failed to spawn generation thread"
                            }),
                        }
                    }
                }
            }
            "generate_skybox" => {
                let prompt = args
                    .get("prompt")
                    .and_then(|v| v.as_str())
                    .unwrap_or("skybox")
                    .to_string();

                let quality = args
                    .get("quality")
                    .and_then(|v| v.as_str())
                    .unwrap_or("high")
                    .to_string();

                let seed = args.get("seed").and_then(|v| v.as_u64());

                log::info!("Generating skybox '{}' (quality: {})", prompt, quality);

                // Clone for use after spawn
                let prompt_for_response = prompt.clone();

                // Try to generate skybox using local Stable Diffusion service
                match std::thread::spawn(move || {
                    generate_skybox_blocking(&prompt, &quality, seed)
                }).join() {
                    Ok(Ok((asset_id, file_path))) => {
                        IpcResponse {
                            id,
                            success: true,
                            result: json!({
                                "generated": true,
                                "asset_id": asset_id,
                                "file_path": file_path,
                                "prompt": prompt_for_response,
                                "dimensions": [2048, 1024]
                            }),
                        }
                    }
                    Ok(Err(e)) => {
                        log::error!("Skybox generation failed: {}", e);
                        IpcResponse {
                            id,
                            success: false,
                            result: json!({
                                "generated": false,
                                "error": format!("Skybox generation failed: {}", e)
                            }),
                        }
                    }
                    Err(_) => {
                        IpcResponse {
                            id,
                            success: false,
                            result: json!({
                                "generated": false,
                                "error": "Failed to spawn generation thread"
                            }),
                        }
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

fn generate_texture_blocking(
    prompt: &str,
    width: u32,
    height: u32,
    quality: &str,
    seed: Option<u64>,
) -> Result<(String, String)> {
    // Get or create asset generator
    let local_client = LocalClient::localhost(7860, 300); // Default Stable Diffusion web UI port
    let cache_dir = "./generated_assets";
    let cache = AssetCache::new(cache_dir)?;
    let generator = AssetGenerator::new(Box::new(local_client), cache)?;

    // Create generation request
    let request = TextureGenerationRequest {
        prompt: prompt.to_string(),
        negative_prompt: Some("blurry, low quality, distorted, ugly".to_string()),
        width,
        height,
        steps: match quality {
            "fast" => 20,
            "standard" => 35,
            "high" => 50,
            "best" => 75,
            _ => 50,
        },
        guidance_scale: match quality {
            "fast" => 5.0,
            "standard" => 7.0,
            "high" => 7.5,
            "best" => 8.5,
            _ => 7.5,
        },
        seed,
        use_cache: true,
    };

    // Run async generation in blocking context
    let rt = tokio::runtime::Runtime::new()?;
    let asset = rt.block_on(generator.generate_texture(&request))?;

    Ok((asset.metadata.id.clone(), asset.metadata.file_path.clone()))
}

fn generate_skybox_blocking(
    prompt: &str,
    quality: &str,
    seed: Option<u64>,
) -> Result<(String, String)> {
    // Get or create asset generator
    let local_client = LocalClient::localhost(7860, 300); // Default Stable Diffusion web UI port
    let cache_dir = "./generated_assets";
    let cache = AssetCache::new(cache_dir)?;
    let generator = AssetGenerator::new(Box::new(local_client), cache)?;

    // Run async generation in blocking context
    let rt = tokio::runtime::Runtime::new()?;
    let asset = rt.block_on(generator.generate_skybox(prompt, seed))?;

    Ok((asset.metadata.id.clone(), asset.metadata.file_path.clone()))
}
