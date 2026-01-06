# MCP Tools Reference

Complete reference for Model Context Protocol (MCP) tools available in Causality Engine.

## Scene Management

### save_scene
Save the current scene to a file.

**Parameters:**
- `file_path` (string, required): Path to save the scene file (e.g., `assets/scenes/my_scene.ron`)

**Example:**
```json
{
  "name": "save_scene",
  "arguments": {
    "file_path": "assets/scenes/my_level.ron"
  }
}
```

**Returns:** Success message with entity count

---

### load_scene
Load a scene from a file.

**Parameters:**
- `file_path` (string, required): Path to the scene file to load (e.g., `assets/scenes/castle.ron`)

**Example:**
```json
{
  "name": "load_scene",
  "arguments": {
    "file_path": "assets/scenes/castle.ron"
  }
}
```

**Returns:** Success message with scene name and entity count

---

### get_scene_info
Get information about the current scene.

**Parameters:** None

**Example:**
```json
{
  "name": "get_scene_info",
  "arguments": {}
}
```

**Returns:** Scene entity count and list of entity names

---

## Entity Management

### create_entity
Create a new entity in the scene.

**Parameters:**
- `name` (string, required): Name of the entity
- `position` (array[3], optional): Position [x, y, z] (default: [0, 0, 0])

**Example:**
```json
{
  "name": "create_entity",
  "arguments": {
    "name": "Player",
    "position": [0, 1, 0]
  }
}
```

---

### list_entities
List all entities in the scene.

**Parameters:** None

**Example:**
```json
{
  "name": "list_entities",
  "arguments": {}
}
```

**Returns:** List of entity names

---

### get_entity_info
Get detailed information about an entity.

**Parameters:**
- `entity_name` (string, required): Name of the entity

**Example:**
```json
{
  "name": "get_entity_info",
  "arguments": {
    "entity_name": "Player"
  }
}
```

**Returns:** Entity position, rotation, and scale

---

### delete_entity
Delete an entity from the scene.

**Parameters:**
- `entity_name` (string, required): Name of the entity to delete

**Example:**
```json
{
  "name": "delete_entity",
  "arguments": {
    "entity_name": "OldEntity"
  }
}
```

---

### set_transform
Set an entity's transform (position, rotation, scale).

**Parameters:**
- `entity_name` (string, required): Name of the entity to modify
- `position` (array[3], optional): Position [x, y, z]
- `scale` (array[3], optional): Scale [x, y, z]

**Example:**
```json
{
  "name": "set_transform",
  "arguments": {
    "entity_name": "Player",
    "position": [5, 0, 10],
    "scale": [1.5, 1.5, 1.5]
  }
}
```

---

## Physics Components

### add_rigidbody
Add a physics rigid body component to an entity.

**Parameters:**
- `entity_name` (string, required): Name of the entity
- `body_type` (string, optional): Body type: `dynamic`, `kinematic`, or `static` (default: `dynamic`)
- `mass` (number, optional): Mass for dynamic bodies (default: 1.0)

**Example:**
```json
{
  "name": "add_rigidbody",
  "arguments": {
    "entity_name": "Crate",
    "body_type": "dynamic",
    "mass": 10.0
  }
}
```

---

### add_collider
Add a physics collider component to an entity.

**Parameters:**
- `entity_name` (string, required): Name of the entity
- `shape_type` (string, optional): Shape: `box`, `sphere`, or `capsule` (default: `box`)
- `size` (array[3], optional): Size [x, y, z] for box collider
- `radius` (number, optional): Radius for sphere and capsule colliders (default: 0.5)
- `height` (number, optional): Height for capsule collider (default: 1.0)

**Example:**
```json
{
  "name": "add_collider",
  "arguments": {
    "entity_name": "Crate",
    "shape_type": "box",
    "size": [1.0, 1.0, 1.0]
  }
}
```

---

## Scripting

### add_script
Add or update a Rhai script on an entity.

**Parameters:**
- `entity_name` (string, required): Name of the entity
- `script` (string, required): Rhai script code

**Example:**
```json
{
  "name": "add_script",
  "arguments": {
    "entity_name": "Player",
    "script": "fn update(delta) { print(\\\"Player updated: \\\" + delta); }"
  }
}
```

---

## Asset Management

### load_model
Load a 3D model (GLTF) into the scene.

**Parameters:**
- `entity_name` (string, required): Name for the new entity
- `model_path` (string, required): Path to the GLTF model file
- `position` (array[3], optional): Position [x, y, z]

**Example:**
```json
{
  "name": "load_model",
  "arguments": {
    "entity_name": "Character",
    "model_path": "assets/models/character.gltf",
    "position": [0, 0, 0]
  }
}
```

---

### generate_texture
Generate a texture from a text prompt using AI (Stable Diffusion).

**Parameters:**
- `prompt` (string, required): Text description of the texture to generate
- `width` (integer, optional): Texture width in pixels (default: 512, must be multiple of 64)
- `height` (integer, optional): Texture height in pixels (default: 512, must be multiple of 64)
- `quality` (string, optional): Quality level: `fast`, `standard`, `high`, or `best` (default: `high`)
- `seed` (integer, optional): Random seed for reproducibility

**Example:**
```json
{
  "name": "generate_texture",
  "arguments": {
    "prompt": "seamless tileable brick wall texture pattern",
    "width": 1024,
    "height": 1024,
    "quality": "high"
  }
}
```

**See also:** `TEXTURE_PROMPT_GUIDE.md` for best practices

---

### generate_skybox
Generate a 360-degree skybox from a text prompt using AI.

**Parameters:**
- `prompt` (string, required): Text description of the skybox environment
- `quality` (string, optional): Quality level: `fast`, `standard`, `high`, or `best` (default: `high`)
- `seed` (integer, optional): Random seed for reproducibility

**Example:**
```json
{
  "name": "generate_skybox",
  "arguments": {
    "prompt": "sunset over mountains with purple clouds",
    "quality": "high"
  }
}
```

---

## Implementation Notes

### IPC Communication
The MCP server communicates with the editor via file-based IPC:
- Commands written to: `/tmp/game-engine-mcp-command.json`
- Responses read from: `/tmp/game-engine-mcp-response.json`
- Timeout: 5 seconds

### Scene File Format
Scenes are saved in RON (Rusty Object Notation) format for human readability:
- Location: `assets/scenes/*.ron`
- Contains: Entity hierarchy, transforms, components
- See: `assets/scenes/castle.ron` for example

### Currently Serialized Components
- MeshRenderer (mesh path, material path)
- Camera (FOV, near/far planes)
- Light (type, color, intensity)

**Note:** Physics components (RigidBody, Collider) are not yet serialized in scene files. They must be added programmatically or via MCP tools after loading a scene.

## Version History

- v0.2.0 (2026-01-06): Added `save_scene` and `load_scene` tools
- v0.1.0 (2025-01-05): Initial MCP server with 12 tools
