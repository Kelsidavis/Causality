# Causality Engine

A modern, modular 3D game engine built from scratch in Rust with native desktop support, physics simulation, scripting, and AI integration via MCP (Model Context Protocol).

## Features

### Core Engine

- **Modern 3D Rendering** (wgpu)
  - PBR-style lighting with texture mapping
  - **Texture rendering with AI-generated assets**
  - **Push constants for efficient multi-object rendering**
  - Multi-mesh rendering with frustum culling
  - Depth testing and shadows
  - Custom shaders (WGSL)
  - LOD (Level of Detail) system
  - Skybox support

- **Scene Management**
  - Entity-component system
  - Transform hierarchy (parent-child relationships)
  - World matrix calculations
  - Scene serialization support (RON format)

- **Asset Pipeline**
  - GLTF model loading
  - **Texture loading and management system**
  - **GPU texture upload with automatic format conversion**
  - **AI asset generation via Stable Diffusion integration**
  - Procedural mesh generation (cube, plane, colored meshes)
  - Asset caching system
  - **Hot-reload for textures, models, and scripts**

- **3D Physics** (Rapier3D)
  - Rigid body dynamics (dynamic, kinematic, static)
  - Collision detection
  - Multiple collider shapes (box, sphere, capsule, cylinder)
  - Physics-scene synchronization
  - **Ragdoll physics system**
  - Constraints and joints
  - Configurable gravity and physics parameters

- **Scripting** (Rhai)
  - Entity-attached scripts
  - Update loop integration
  - Rich API (Vec3, Quat, math functions)
  - **Real-time script hot-reload**
  - Transform manipulation from scripts

- **Audio System**
  - 3D spatial audio
  - Multiple sound sources
  - Volume and position control

- **Particle System**
  - GPU-accelerated particles
  - Emitter configurations

### Editor

- **Live Viewport**
  - Real-time 3D scene preview
  - **Working egui UI overlay**
  - Orbit, pan, zoom camera controls

- **UI Panels**
  - Hierarchy panel (entity tree view)
  - Inspector panel (transform editing)
  - Console panel (logs with color coding)
  - Menu bar (File, Edit, View, Help)

- **Hot Reload**
  - Automatic asset reloading on file changes
  - Script hot-swapping without restart
  - Texture and model live updates

### AI Integration

- **MCP Server** (Model Context Protocol)
  - **Live file-based IPC connection**
  - Claude Code integration ready
  - 8 MCP tools for engine control
  - JSON-RPC over stdio
  - Real-time communication with editor

- **Stable Diffusion Integration**
  - **AI-powered texture generation via ComfyUI**
  - Local Stable Diffusion server support
  - Procedural asset creation for terrains, materials, and structures
  - Network-wide serving for collaborative workflows
  - Generated assets automatically loaded and rendered

### Game UI Framework

- **Widget System**
  - Label, Button, HealthBar, ProgressBar
  - Panel, Image, Slider, TextInput
  - Canvas-based drawing
  - Layout management (horizontal/vertical)

### Performance Features

- **Frustum Culling** - Skip rendering off-screen objects
- **LOD System** - Level-of-detail management
- **Build System** - Package games as standalone executables

## Quick Start

### Prerequisites

- Rust 1.70+ (with cargo)
- Vulkan/Metal/DirectX 12 compatible GPU

### Build and Run

```bash
# Run the editor
cargo run --bin editor

# Run the MCP server (for Claude Code integration)
cargo run --bin engine-mcp-server
```

### Controls

**Camera (in viewport):**
- Right-click + drag: Orbit camera
- Middle-click + drag: Pan camera
- Mouse wheel: Zoom in/out
- Escape: Exit

**Demo Scene:**
- **Castle and Countryside** - Complete medieval scene with AI-generated textures
  - 4 castle walls with defensive towers
  - Central keep fortress
  - Moat surrounding the castle
  - Grass terrain with rolling hills
  - 19 entities rendered simultaneously with push constants
  - Textured with Stable Diffusion generated assets (stone, grass, water)

## MCP Integration

The Causality Engine includes a fully functional MCP server that allows Claude Code to control the engine in real-time.

### Available MCP Tools

**Core Operations**:
1. **create_entity** - Create new entities with optional position
2. **list_entities** - Query all entities in the scene
3. **get_entity_info** - Get detailed entity information (position, rotation, scale)
4. **delete_entity** - Remove entities from the scene
5. **set_transform** - Modify entity position and scale
6. **get_scene_info** - Query scene metadata and statistics

**Asset & Model Loading**:
7. **load_model** - Load GLTF 3D models and create entities with them
8. **add_script** - Attach Rhai scripts to entities for custom behavior

**Physics Components**:
9. **add_rigidbody** - Add physics simulation (dynamic, kinematic, static)
10. **add_collider** - Add collision shapes (box, sphere, capsule)

All tools support real-time scene control with full error handling and feedback.

### MCP Configuration

Add to your Claude Code MCP config:

```json
{
  "causality-engine": {
    "command": "cargo",
    "args": ["run", "--bin", "engine-mcp-server"],
    "cwd": "/path/to/causality-engine"
  }
}
```

See [MCP_TOOLS_GUIDE.md](MCP_TOOLS_GUIDE.md) for complete tool documentation and [MCP_GUIDE.md](MCP_GUIDE.md) for configuration details.

## Project Structure

```
causality-engine/
├── crates/
│   ├── engine-core/          # Core systems (timing, input, build system)
│   ├── engine-render/        # wgpu rendering, camera, culling, LOD
│   │   ├── gpu_texture.rs    # GPU texture management
│   │   ├── texture_manager.rs # Texture loading and caching
│   │   └── shaders/          # WGSL shaders (PBR, post-processing)
│   ├── engine-physics/       # Rapier3D wrapper, ragdoll physics
│   ├── engine-scripting/     # Rhai runtime, API bindings, hot-reload
│   ├── engine-assets/        # GLTF loading, texture loading, hot-reload
│   ├── engine-scene/         # Entity system, scene graph
│   ├── engine-audio/         # 3D spatial audio system
│   ├── engine-particles/     # Particle system
│   ├── engine-ui/            # Game UI framework (widgets, canvas)
│   ├── engine-editor/        # Editor application with egui UI
│   └── engine-mcp-server/    # MCP server for Claude Code integration
├── generated_assets/         # AI-generated textures and models
│   └── textures/            # Stable Diffusion generated textures
├── mcp-config.json           # MCP server configuration
├── MCP_GUIDE.md              # MCP usage documentation
├── TEXTURE_SYSTEM.md         # Texture system documentation
└── README.md                 # This file
```

## Development Status

### Completed Features ✅

- ✅ **Phase 1**: Foundation (wgpu, camera, rendering)
- ✅ **Phase 2**: Scene System (entities, GLTF, assets)
- ✅ **Phase 3**: Physics (Rapier3D, collisions)
- ✅ **Phase 4**: Scripting (Rhai integration)
- ✅ **Phase 5**: Editor Polish (UI panels, camera controls)
- ✅ **Phase 6**: MCP Integration (Claude Code protocol)
- ✅ **Phase 7**: Hot Reload (asset and script hot-swapping)
- ✅ **Priority 1**: Critical fixes
  - ✅ egui rendering integration
  - ✅ MCP server live connection
  - ✅ Full asset hot-reload system
- ✅ **Priority 3**: Ragdoll physics
- ✅ **Priority 5**: Frustum culling, LOD system
- ✅ **Priority 6**: Build system, input system, game UI framework
- ✅ **Phase 8**: Texture rendering system
  - ✅ GPU texture management
  - ✅ Texture loading with automatic format conversion
  - ✅ Push constants for multi-object rendering
  - ✅ AI-generated asset integration (Stable Diffusion)
  - ✅ Castle and countryside demo scene

### In Progress 📋

- **Phase 9**: Advanced rendering features
  - Shadow mapping
  - Post-processing effects (bloom, tone mapping)
  - Advanced PBR materials
  - Normal mapping

## Technology Stack

| Component | Technology | Version |
|-----------|-----------|---------|
| Graphics API | wgpu | 27.0 |
| Physics | Rapier3D | 0.22 |
| Scripting | Rhai | 1.19 |
| UI | egui + egui-wgpu | 0.33 |
| Windowing | winit | 0.30 |
| Math | glam | 0.29 |
| Asset Format | GLTF, RON | 1.4, 0.8 |
| File Watching | notify | 7.0 |

## Scripting Example

```rust
// player_controller.rhai
fn update(ctx) {
    // Move forward when W is pressed
    if input.key_pressed("W") {
        let forward = vec3(0.0, 0.0, -5.0);
        ctx.position = ctx.position + forward * ctx.dt;
    }

    // Jump when Space is pressed
    if input.key_pressed("Space") {
        ctx.apply_force(vec3(0.0, 500.0, 0.0));
    }

    ctx
}
```

## License

Copyright © 2025 Causality Engine Contributors

## Contributing

This is a personal project, but feedback and suggestions are welcome!
