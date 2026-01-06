# Causality Engine MCP Tools Guide

This guide describes all available MCP tools for controlling the Causality Engine via Claude Code.

## Overview

The Causality Engine exposes a comprehensive set of tools through the Model Context Protocol (MCP) that allow Claude to:

- **Create and manage entities** in the scene
- **Transform objects** (position, rotation, scale)
- **Add physics** (rigidbodies and colliders)
- **Attach scripts** (Rhai scripting language)
- **Load 3D models** (GLTF format)
- **Query scene state** (list entities, get properties)

## Tool Reference

### 1. create_entity

**Description**: Create a new entity in the scene

**Arguments**:
- `name` (string, required): Name of the entity
- `position` (array, optional): Position [x, y, z] (default: [0, 0, 0])

**Returns**:
- `entity_id`: Unique identifier for the created entity
- `name`: Entity name

**Example**:
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

### 2. list_entities

**Description**: List all entities currently in the scene

**Arguments**: None

**Returns**:
- `entities`: Array of entity names
- `count`: Total number of entities

**Example**:
```json
{
  "name": "list_entities",
  "arguments": {}
}
```

---

### 3. get_entity_info

**Description**: Get detailed information about a specific entity

**Arguments**:
- `entity_name` (string, required): Name of the entity to query

**Returns**:
- `entity_id`: Unique identifier
- `name`: Entity name
- `position`: [x, y, z] coordinates
- `rotation`: [x, y, z, w] quaternion
- `scale`: [x, y, z] scale values

**Example**:
```json
{
  "name": "get_entity_info",
  "arguments": {
    "entity_name": "Player"
  }
}
```

---

### 4. set_transform

**Description**: Modify an entity's position and/or scale

**Arguments**:
- `entity_name` (string, required): Name of the entity
- `position` (array, optional): New position [x, y, z]
- `scale` (array, optional): New scale [x, y, z]

**Returns**:
- `updated`: Boolean indicating success
- `entity_name`: The entity that was updated

**Example**:
```json
{
  "name": "set_transform",
  "arguments": {
    "entity_name": "Player",
    "position": [5, 2, -3],
    "scale": [2, 2, 2]
  }
}
```

---

### 5. delete_entity

**Description**: Remove an entity from the scene

**Arguments**:
- `entity_name` (string, required): Name of the entity to delete

**Returns**:
- `deleted`: Boolean indicating success
- `name`: Name of deleted entity

**Example**:
```json
{
  "name": "delete_entity",
  "arguments": {
    "entity_name": "Enemy1"
  }
}
```

---

### 6. load_model

**Description**: Load a 3D model (GLTF format) and create an entity with it

**Arguments**:
- `entity_name` (string, required): Name for the new entity
- `model_path` (string, required): Path to GLTF model file
- `position` (array, optional): Position [x, y, z] (default: [0, 0, 0])

**Returns**:
- `model_loaded`: Boolean indicating success
- `entity_name`: Entity name
- `model_path`: Path that was loaded
- `position`: Position where model was placed

**Example**:
```json
{
  "name": "load_model",
  "arguments": {
    "entity_name": "Knight",
    "model_path": "models/knight.gltf",
    "position": [0, 0, 5]
  }
}
```

---

### 7. add_script

**Description**: Attach a Rhai script to an entity

**Arguments**:
- `entity_name` (string, required): Name of the entity
- `script` (string, required): Rhai script source code

**Returns**:
- `script_added`: Boolean indicating success
- `entity_name`: Entity name
- `script_size`: Size of script in bytes

**Example**:
```json
{
  "name": "add_script",
  "arguments": {
    "entity_name": "Player",
    "script": "fn update(ctx) {\n  let forward = vec3(0.0, 0.0, -5.0);\n  ctx.position = ctx.position + forward * ctx.dt;\n  ctx\n}"
  }
}
```

**Script API**:
- `ctx.position`: Current entity position (Vec3)
- `ctx.rotation`: Current rotation (Quat)
- `ctx.dt`: Delta time since last frame (f32)
- `ctx.apply_force(force: Vec3)`: Apply physics force
- `vec3(x, y, z)`: Create a vector
- Standard Rhai functions available

---

### 8. add_rigidbody

**Description**: Add physics simulation to an entity via RigidBody component

**Arguments**:
- `entity_name` (string, required): Name of the entity
- `body_type` (string, optional): Body type - "dynamic", "kinematic", or "static" (default: "dynamic")
- `mass` (number, optional): Mass for dynamic bodies (default: 1.0)

**Body Types**:
- `dynamic`: Affected by forces and gravity
- `kinematic`: Can be moved programmatically, not affected by forces
- `static`: Never moves

**Returns**:
- `rigidbody_added`: Boolean indicating success
- `entity_name`: Entity name
- `body_type`: Type of body added
- `mass`: Mass value

**Example**:
```json
{
  "name": "add_rigidbody",
  "arguments": {
    "entity_name": "Ball",
    "body_type": "dynamic",
    "mass": 2.5
  }
}
```

---

### 9. add_collider

**Description**: Add collision detection to an entity via Collider component

**Arguments**:
- `entity_name` (string, required): Name of the entity
- `shape_type` (string, optional): Collider shape - "box", "sphere", or "capsule" (default: "box")
- `size` (array, optional): [x, y, z] half-extents for box (default: [0.5, 0.5, 0.5])
- `radius` (number, optional): Radius for sphere/capsule (default: 0.5)
- `height` (number, optional): Full height for capsule (default: 1.0)

**Shape Details**:
- `box`: Defined by half-extents in x, y, z
- `sphere`: Defined by radius
- `capsule`: Cylinder with hemispheres on ends (height + 2*radius = total height)

**Returns**:
- `collider_added`: Boolean indicating success
- `entity_name`: Entity name
- `shape_type`: Shape that was added
- `size`: Size values (for box)
- `radius`: Radius value (for sphere/capsule)

**Example**:
```json
{
  "name": "add_collider",
  "arguments": {
    "entity_name": "Ball",
    "shape_type": "sphere",
    "radius": 0.5
  }
}
```

---

### 10. get_scene_info

**Description**: Get overall information about the current scene

**Arguments**: None

**Returns**:
- `entity_count`: Total number of entities in scene
- `entities`: Array of entity names
- `physics`: Physics enabled status
- `scripting`: Scripting enabled status

**Example**:
```json
{
  "name": "get_scene_info",
  "arguments": {}
}
```

---

## Common Workflows

### Creating a Physics-Enabled Entity

```python
# 1. Create entity
create_entity(name="Box", position=[0, 5, 0])

# 2. Load model (or it's created with basic mesh)
load_model(entity_name="Box", model_path="models/cube.gltf")

# 3. Add physics
add_rigidbody(entity_name="Box", body_type="dynamic", mass=1.0)
add_collider(entity_name="Box", shape_type="box", size=[0.5, 0.5, 0.5])
```

### Creating a Scripted Character

```python
# 1. Create entity
create_entity(name="Player", position=[0, 1, 0])

# 2. Load character model
load_model(entity_name="Player", model_path="models/character.gltf")

# 3. Add physics for movement
add_rigidbody(entity_name="Player", body_type="dynamic", mass=80.0)
add_collider(entity_name="Player", shape_type="capsule", height=1.8, radius=0.3)

# 4. Attach control script
add_script(
  entity_name="Player",
  script="""
    fn update(ctx) {
      if input.key_pressed("W") {
        ctx.apply_force(vec3(0, 0, -50))
      }
      if input.key_pressed("Space") {
        ctx.apply_force(vec3(0, 500, 0))
      }
      ctx
    }
  """
)
```

### Querying Scene State

```python
# Get all entities
list_entities()

# Get specific entity details
get_entity_info(entity_name="Player")

# Get scene overview
get_scene_info()
```

---

## Error Handling

All tools return responses with success status:

```json
{
  "success": true,
  "result": { /* tool-specific results */ }
}
```

On error:

```json
{
  "success": false,
  "result": { "error": "Error description" }
}
```

---

## Tips and Best Practices

1. **Entity Naming**: Use descriptive names like "Player", "Enemy1", "Platform" for easy identification
2. **Component Order**: Add rigidbody before collider for consistent initialization
3. **Physics Mass**: Use realistic mass values (1-100 kg for most objects)
4. **Collider Sizes**: Match collider dimensions to your model's actual size
5. **Script Debugging**: Use `log` in Rhai scripts to debug behavior
6. **Asset Paths**: Ensure GLTF paths are relative to the project root or use absolute paths

---

## Integration with Claude Code

Use these tools naturally in conversation:

**"Create a scene with a falling cube"**:
```
Create entity "Cube" at [0, 5, 0]
Load GLTF model for it
Add dynamic rigidbody with mass 1.0
Add box collider
```

**"Make the player jump"**:
```
Add script to "Player" with jump logic on Space key
Apply upward force when Space is pressed
```

**"What objects are in the scene?"**:
```
List all entities and their positions
```

---

## Limitations and Future Work

- Model loading requires pre-existing GLTF files (procedural generation coming soon)
- Script parameters are limited to the basic API (expansion planned)
- No animation support yet (scheduled for Phase 9)
- Constraint/joint physics coming in advanced phase
- Audio control tools in development

---

For more information, see [MCP_GUIDE.md](MCP_GUIDE.md) and [README.md](README.md)
