# Game Engine MCP Server - Usage Guide

## Overview

The Game Engine MCP (Model Context Protocol) Server allows Claude Code to control the game engine programmatically. You can create entities, modify transforms, add scripts, and more—all through conversational AI.

## Setup

### 1. Configure Claude Code

Add the MCP server to your Claude Code configuration file:

**Location**: `~/.config/claude/mcp_config.json` (Linux/Mac) or `%APPDATA%\Claude\mcp_config.json` (Windows)

```json
{
  "mcpServers": {
    "game-engine": {
      "command": "cargo",
      "args": ["run", "--bin", "engine-mcp-server", "--quiet"],
      "cwd": "/home/k/game-engine",
      "env": {
        "RUST_LOG": "info"
      }
    }
  }
}
```

**Note**: Update the `cwd` path to match your actual game engine directory.

### 2. Restart Claude Code

After adding the configuration, restart Claude Code to load the MCP server.

### 3. Verify Connection

Ask Claude: "What tools do you have available for the game engine?"

Claude should list all available MCP tools.

## Available Tools

### create_entity
Create a new entity in the scene.

**Parameters:**
- `name` (string, required): Name of the entity
- `position` (array, optional): Position [x, y, z], defaults to [0, 0, 0]

**Example:**
```
User: "Create a player entity at position (0, 5, 0)"
Claude: [Uses create_entity tool]
```

### set_transform
Set an entity's transform (position, rotation, scale).

**Parameters:**
- `entity_name` (string, required): Name of the entity to modify
- `position` (array, optional): Position [x, y, z]
- `scale` (array, optional): Scale [x, y, z]

**Example:**
```
User: "Move the 'Player' entity to (10, 0, 5)"
Claude: [Uses set_transform tool]
```

### list_entities
List all entities in the current scene.

**Example:**
```
User: "Show me all entities in the scene"
Claude: [Uses list_entities tool]
```

### get_entity_info
Get detailed information about a specific entity.

**Parameters:**
- `entity_name` (string, required): Name of the entity

**Example:**
```
User: "What's the position of 'Falling Cube'?"
Claude: [Uses get_entity_info tool]
```

### delete_entity
Delete an entity from the scene.

**Parameters:**
- `entity_name` (string, required): Name of the entity to delete

**Example:**
```
User: "Delete the 'Ground Plane' entity"
Claude: [Uses delete_entity tool]
```

### add_script
Add or update a Rhai script on an entity.

**Parameters:**
- `entity_name` (string, required): Name of the entity
- `script` (string, required): Rhai script code

**Example:**
```
User: "Add a script to 'Player' that makes it jump when pressing space"
Claude: [Uses add_script tool with appropriate Rhai code]
```

### load_model
Load a 3D model (GLTF) into the scene.

**Parameters:**
- `entity_name` (string, required): Name for the new entity
- `model_path` (string, required): Path to the GLTF model file
- `position` (array, optional): Position [x, y, z]

**Example:**
```
User: "Load the character.gltf model as 'Hero'"
Claude: [Uses load_model tool]
```

### get_scene_info
Get information about the current scene.

**Example:**
```
User: "Tell me about the current scene"
Claude: [Uses get_scene_info tool]
```

## Example Conversations

### Creating a Simple Scene

```
You: "Create a new scene with a ground plane and a player character"

Claude: I'll create a scene for you.
[Uses create_entity: name="Ground", position=[0, 0, 0]]
[Uses create_entity: name="Player", position=[0, 1, 0]]

Created a ground plane at the origin and a player character standing on it at position (0, 1, 0).

You: "Add physics to the player so it falls"

Claude: I'll add a script with physics behavior.
[Uses add_script: entity_name="Player", script="...rigidbody code..."]

Added a physics script to the Player. It will now respond to gravity.
```

### Modifying Entities

```
You: "List all entities"

Claude: [Uses list_entities]
The scene contains:
- Falling Cube
- Ground Plane
- Falling Small Cube

You: "Move the Falling Cube to (5, 10, 0)"

Claude: [Uses set_transform: entity_name="Falling Cube", position=[5, 10, 0]]
Moved 'Falling Cube' to position (5, 10, 0).

You: "What's the state of the Falling Cube now?"

Claude: [Uses get_entity_info: entity_name="Falling Cube"]
Entity 'Falling Cube' info:
Position: [5, 10, 0]
Rotation: [0, 0, 0, 1]
Scale: [1, 1, 1]
```

### Adding Scripts

```
You: "Create a rotating platform"

Claude: [Uses create_entity: name="Platform", position=[0, 0, 0]]
[Uses add_script: entity_name="Platform", script="
fn update(ctx) {
    let rotation_speed = 1.0;
    let angle = ctx.dt * rotation_speed;
    let rotation_delta = quat_from_rotation_y(angle);
    ctx.rotation = ctx.rotation * rotation_delta;
    ctx
}
"]

Created a platform that rotates around the Y axis.
```

## Current Limitations

**Note**: This is Phase 6 implementation. The MCP server currently provides **simulated responses** because:

1. **No IPC Yet**: The MCP server doesn't yet communicate with the running editor
2. **Standalone Mode**: The server can be tested independently but won't modify the actual game scene
3. **Proof of Concept**: All tools return success messages but don't execute actual operations

### Future Enhancements (Phase 7+)

- Add IPC (Inter-Process Communication) between MCP server and editor
- Real-time scene manipulation from Claude Code
- Screenshot capture tool
- Camera control tools
- Asset browsing and loading
- Scene save/load operations

## Testing the MCP Server

You can test the MCP server independently:

```bash
cd /home/k/game-engine
cargo run --bin engine-mcp-server
```

Then send JSON-RPC messages via stdin:

```json
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}
{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"list_entities","arguments":{}}}
```

Press Ctrl+D to end input and see responses.

## Troubleshooting

### MCP Server Not Connecting

1. Check the path in `mcp_config.json` matches your game engine directory
2. Ensure cargo is in your PATH
3. Check logs: `RUST_LOG=debug cargo run --bin engine-mcp-server`

### Tools Not Working

1. Verify the server is running: Check Claude Code's MCP connection status
2. Look for error messages in the MCP server output
3. Ensure the JSON schema matches the tool requirements

## Architecture

```
┌─────────────┐         JSON-RPC         ┌──────────────────┐
│             │◄─────── over stdio ───────►│                  │
│ Claude Code │                           │  MCP Server      │
│             │         (Phase 6)         │  (Standalone)    │
└─────────────┘                           └──────────────────┘
                                                    │
                                                    │ IPC
                                                    │ (Phase 7)
                                                    ▼
                                          ┌──────────────────┐
                                          │                  │
                                          │  Game Engine     │
                                          │  Editor          │
                                          │                  │
                                          └──────────────────┘
```

## Next Steps

To complete full integration (Phase 7):

1. Implement IPC using crossbeam-channel or similar
2. Add message passing between MCP server and editor
3. Update editor to handle MCP commands
4. Add screenshot/viewport capture capability
5. Implement scene persistence

For now, the MCP server demonstrates the protocol and tool definitions, ready for Claude Code integration!
