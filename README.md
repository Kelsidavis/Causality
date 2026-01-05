# 3D Game Engine in Rust

A modern, modular 3D game engine built from scratch in Rust with native desktop support, physics simulation, scripting, and AI integration via MCP (Model Context Protocol).

## Features

### ✅ Completed (Phases 1-6)

- **Modern 3D Rendering** (wgpu)
  - PBR-style lighting
  - Multi-mesh rendering
  - Depth testing
  - Custom shaders (WGSL)

- **Scene Management**
  - Entity-component system
  - Transform hierarchy (parent-child relationships)
  - World matrix calculations
  - Scene serialization support (RON format)

- **Asset Pipeline**
  - GLTF model loading
  - Texture loading
  - Procedural mesh generation (cube, plane)
  - Asset caching system

- **3D Physics** (Rapier3D)
  - Rigid body dynamics (dynamic, kinematic, static)
  - Collision detection
  - Multiple collider shapes (box, sphere, capsule, cylinder)
  - Physics-scene synchronization
  - Configurable gravity and physics parameters

- **Scripting** (Rhai)
  - Entity-attached scripts
  - Update loop integration
  - Rich API (Vec3, Quat, math functions)
  - Script hot-reload capability
  - Transform manipulation from scripts

- **Editor UI** (Phase 5 - UI Written, Rendering Pending)
  - Hierarchy panel (entity tree view)
  - Inspector panel (transform editing)
  - Console panel (logs with color coding)
  - Menu bar (File, Edit, View, Help)
  - Viewport camera controls (orbit, pan, zoom)

  *Note: UI panels implemented but not rendered due to egui-wgpu compatibility issue*

- **MCP Integration** (Phase 6 - Protocol Complete)
  - Model Context Protocol server
  - Claude Code integration ready
  - 8 MCP tools for engine control
  - JSON-RPC over stdio
  - Comprehensive documentation

## Quick Start

### Prerequisites

- Rust 1.70+ (with cargo)
- OpenGL/Vulkan/Metal/DirectX 12 compatible GPU

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
- Watch two cubes fall and collide with physics
- Observe scripted rotation on the cubes
- Gravity simulation at -9.81 m/s²

## MCP Tools Available

1. **create_entity** - Create new entities
2. **set_transform** - Modify entity transforms
3. **list_entities** - Query all entities
4. **get_entity_info** - Get entity details
5. **delete_entity** - Remove entities
6. **add_script** - Attach Rhai scripts
7. **load_model** - Load GLTF models
8. **get_scene_info** - Query scene metadata

See [MCP_GUIDE.md](MCP_GUIDE.md) for complete MCP documentation.

## Project Structure

```
game-engine/
├── crates/
│   ├── engine-core/          # Core systems (timing, math utilities)
│   ├── engine-render/        # wgpu rendering, camera, shaders
│   ├── engine-physics/       # Rapier3D wrapper, physics sync
│   ├── engine-scripting/     # Rhai runtime, API bindings
│   ├── engine-assets/        # GLTF loading, asset management
│   ├── engine-scene/         # Entity system, scene graph
│   ├── engine-editor/        # Editor application with UI
│   └── engine-mcp-server/    # MCP server for Claude Code
├── mcp-config.json          # MCP server configuration
├── MCP_GUIDE.md             # MCP usage documentation
└── README.md                # This file
```

## Development Phases

- ✅ **Phase 1**: Foundation (wgpu, camera, rendering)
- ✅ **Phase 2**: Scene System (entities, GLTF, assets)
- ✅ **Phase 3**: Physics (Rapier3D, collisions)
- ✅ **Phase 4**: Scripting (Rhai integration)
- ✅ **Phase 5**: Editor Polish (UI panels, camera controls)
- ✅ **Phase 6**: MCP Integration (Claude Code protocol)
- ✅ **Phase 7**: Hot Reload (script hot-swapping, file watching)
- 📋 **Phase 8**: Advanced Features (shadows, PBR, character controller)

## Technology Stack

| Component | Technology | Version |
|-----------|-----------|---------|
| Graphics API | wgpu | 23.0 |
| Physics | Rapier3D | 0.22 |
| Scripting | Rhai | 1.19 |
| UI | egui | 0.30 |
| Windowing | winit | 0.30 |
| Math | glam | 0.29 |

## License

MIT License
