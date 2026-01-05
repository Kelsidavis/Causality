// IPC (Inter-Process Communication) between editor and MCP server

use anyhow::Result;
use crossbeam_channel::{Receiver, Sender, unbounded};
use serde::{Deserialize, Serialize};
use engine_scene::{entity::EntityId, Scene};
use glam::{Vec3, Quat};

/// Commands that can be sent from MCP server to editor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EditorCommand {
    /// Create a new entity
    CreateEntity {
        name: String,
        position: Vec3,
        model_path: Option<String>,
    },
    /// Set entity transform
    SetTransform {
        entity_id: EntityId,
        position: Option<Vec3>,
        rotation: Option<Quat>,
        scale: Option<Vec3>,
    },
    /// Delete an entity
    DeleteEntity {
        entity_id: EntityId,
    },
    /// Add/update script on entity
    AddScript {
        entity_id: EntityId,
        script_source: String,
    },
    /// Load a model into the scene
    LoadModel {
        path: String,
        position: Vec3,
    },
    /// Get scene information
    GetSceneInfo,
    /// List all entities
    ListEntities,
    /// Get entity details
    GetEntityInfo {
        entity_id: EntityId,
    },
}

/// Responses sent from editor back to MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EditorResponse {
    /// Entity was created successfully
    EntityCreated {
        entity_id: EntityId,
        name: String,
    },
    /// Transform was updated
    TransformUpdated {
        entity_id: EntityId,
    },
    /// Entity was deleted
    EntityDeleted {
        entity_id: EntityId,
    },
    /// Script was added/updated
    ScriptUpdated {
        entity_id: EntityId,
    },
    /// Model was loaded
    ModelLoaded {
        entity_id: EntityId,
    },
    /// Scene information
    SceneInfo {
        entity_count: usize,
        entities: Vec<String>,
    },
    /// List of entities
    EntityList {
        entities: Vec<(EntityId, String)>,
    },
    /// Entity details
    EntityInfo {
        entity_id: EntityId,
        name: String,
        position: Vec3,
        rotation: Quat,
        scale: Vec3,
        has_mesh: bool,
        has_script: bool,
        has_rigidbody: bool,
    },
    /// Error occurred
    Error {
        message: String,
    },
}

/// IPC channel for communication between editor and MCP server
pub struct IpcChannel {
    /// Receiver for commands from MCP server
    pub command_rx: Receiver<EditorCommand>,
    /// Sender for responses to MCP server
    pub response_tx: Sender<EditorResponse>,
}

impl IpcChannel {
    /// Create a new IPC channel pair (editor side and MCP server side)
    pub fn create() -> (IpcChannel, IpcServer) {
        let (command_tx, command_rx) = unbounded();
        let (response_tx, response_rx) = unbounded();

        let editor_channel = IpcChannel {
            command_rx,
            response_tx,
        };

        let server_channel = IpcServer {
            command_tx,
            response_rx,
        };

        (editor_channel, server_channel)
    }

    /// Check for incoming commands (non-blocking)
    pub fn try_recv_command(&self) -> Result<Option<EditorCommand>> {
        match self.command_rx.try_recv() {
            Ok(cmd) => Ok(Some(cmd)),
            Err(crossbeam_channel::TryRecvError::Empty) => Ok(None),
            Err(e) => Err(anyhow::anyhow!("IPC receive error: {}", e)),
        }
    }

    /// Send a response back to MCP server
    pub fn send_response(&self, response: EditorResponse) -> Result<()> {
        self.response_tx.send(response)?;
        Ok(())
    }
}

/// IPC channel for MCP server side
pub struct IpcServer {
    /// Sender for commands to editor
    pub command_tx: Sender<EditorCommand>,
    /// Receiver for responses from editor
    pub response_rx: Receiver<EditorResponse>,
}

impl IpcServer {
    /// Send a command to the editor
    pub fn send_command(&self, command: EditorCommand) -> Result<()> {
        self.command_tx.send(command)?;
        Ok(())
    }

    /// Wait for response from editor (blocking)
    pub fn recv_response(&self) -> Result<EditorResponse> {
        Ok(self.response_rx.recv()?)
    }

    /// Try to receive response (non-blocking)
    pub fn try_recv_response(&self) -> Result<Option<EditorResponse>> {
        match self.response_rx.try_recv() {
            Ok(resp) => Ok(Some(resp)),
            Err(crossbeam_channel::TryRecvError::Empty) => Ok(None),
            Err(e) => Err(anyhow::anyhow!("IPC receive error: {}", e)),
        }
    }
}

/// Execute an editor command on the scene
pub fn execute_command(
    command: EditorCommand,
    scene: &mut Scene,
) -> EditorResponse {
    match command {
        EditorCommand::CreateEntity { name, position, model_path } => {
            let entity_id = scene.create_entity(name.clone());

            if let Some(entity) = scene.get_entity_mut(entity_id) {
                entity.transform.position = position;

                // TODO: Load model if model_path is provided
                if model_path.is_some() {
                    log::warn!("Model loading via IPC not yet implemented");
                }
            }

            EditorResponse::EntityCreated {
                entity_id,
                name,
            }
        }
        EditorCommand::SetTransform { entity_id, position, rotation, scale } => {
            if let Some(entity) = scene.get_entity_mut(entity_id) {
                if let Some(pos) = position {
                    entity.transform.position = pos;
                }
                if let Some(rot) = rotation {
                    entity.transform.rotation = rot;
                }
                if let Some(scl) = scale {
                    entity.transform.scale = scl;
                }
                EditorResponse::TransformUpdated { entity_id }
            } else {
                EditorResponse::Error {
                    message: format!("Entity {:?} not found", entity_id),
                }
            }
        }
        EditorCommand::DeleteEntity { entity_id } => {
            scene.remove_entity(entity_id);
            EditorResponse::EntityDeleted { entity_id }
        }
        EditorCommand::AddScript { entity_id, script_source } => {
            // TODO: Implement script addition via IPC
            log::warn!("Script addition via IPC not yet implemented");
            EditorResponse::ScriptUpdated { entity_id }
        }
        EditorCommand::LoadModel { path, position } => {
            // TODO: Implement model loading via IPC
            log::warn!("Model loading via IPC not yet implemented");
            EditorResponse::Error {
                message: "Model loading not yet implemented".to_string(),
            }
        }
        EditorCommand::GetSceneInfo => {
            let entities: Vec<String> = scene.entities()
                .map(|e| e.name.clone())
                .collect();

            EditorResponse::SceneInfo {
                entity_count: scene.entities().count(),
                entities,
            }
        }
        EditorCommand::ListEntities => {
            let entities: Vec<(EntityId, String)> = scene.entities()
                .map(|entity| (entity.id, entity.name.clone()))
                .collect();

            EditorResponse::EntityList { entities }
        }
        EditorCommand::GetEntityInfo { entity_id } => {
            if let Some(entity) = scene.get_entity(entity_id) {
                EditorResponse::EntityInfo {
                    entity_id,
                    name: entity.name.clone(),
                    position: entity.transform.position,
                    rotation: entity.transform.rotation,
                    scale: entity.transform.scale,
                    has_mesh: false, // TODO: Check for mesh component
                    has_script: false, // TODO: Check for script component
                    has_rigidbody: false, // TODO: Check for rigidbody component
                }
            } else {
                EditorResponse::Error {
                    message: format!("Entity {:?} not found", entity_id),
                }
            }
        }
    }
}
