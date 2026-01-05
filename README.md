# 3D Game Engine in Rust

An advanced 3D game engine built from scratch in Rust with MCP (Model Context Protocol) integration for Claude Code.

## Phase 1: Foundation ✅ COMPLETE

**Status**: Successfully implemented and built!

### What's Working

- **wgpu Rendering**: Cross-platform 3D graphics using wgpu
- **Camera System**: Perspective camera with configurable FOV and positioning
- **3D Cube Rendering**: Colored, rotating cube demonstrating 3D transforms
- **Window Management**: Cross-platform window creation with winit
- **Project Structure**: Complete workspace with 8 crates

### How to Run

```bash
cd /home/k/game-engine
cargo run --release --bin editor
```

**Controls**:
- **ESC**: Exit the application
- The cube rotates automatically

### Architecture

The engine uses a modular crate structure:

```
game-engine/
├── engine-core/        # App lifecycle, timing, input
├── engine-render/      # wgpu renderer, camera, shaders
├── engine-physics/     # Rapier3D integration (Phase 3)
├── engine-scripting/   # Rhai runtime (Phase 4)
├── engine-assets/      # Asset loading (Phase 2)
├── engine-scene/       # Scene graph (Phase 2)
├── engine-editor/      # Editor application
└── engine-mcp-server/  # MCP server (Phase 6)
```

### Technology Stack

- **Rendering**: wgpu (modern cross-platform graphics)
- **Math**: glam (SIMD-accelerated linear algebra)
- **Windowing**: winit (cross-platform window creation)
- **Build**: Cargo workspace with optimized release builds

## Next Steps: Phase 2 - Scene System

Phase 2 will add:
- Entity/component system
- GLTF model loading
- Scene serialization with RON
- Multiple meshes, materials, and lighting

See the full plan at: `~/.claude/plans/zesty-doodling-marshmallow.md`

## Build Information

- Built on: 2026-01-05
- Rust Edition: 2021
- Target: Desktop (Linux/Windows/Mac via wgpu)
- Build Time: ~1.8s (release mode)

## Development Notes

### Current Limitations

- No UI overlay yet (egui integration deferred to later phase)
- Single hardcoded cube (scene system in Phase 2)
- No physics (Phase 3)
- No scripting (Phase 4)

### Performance

The engine runs at native refresh rate (typically 60 FPS) with minimal CPU/GPU usage for the simple cube scene.

## License

TBD
