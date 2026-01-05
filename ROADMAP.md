# Game Engine - Improvement Roadmap

## Priority 1: Fix Known Issues (Critical)

### 1.1 egui Rendering Integration
**Problem**: UI panels are written but not displayed due to egui-wgpu 0.30 + wgpu 23.0 compatibility issue

**Solution Options**:
- **Option A**: Downgrade wgpu to 0.19 (egui-wgpu 0.30 compatible version)
- **Option B**: Wait for egui-wgpu 0.31 release with wgpu 23.0 support
- **Option C**: Implement custom egui renderer integration with proper borrow handling

**Recommended**: Option A for immediate fix, migrate to Option B when available

**Impact**: HIGH - Makes editor fully functional with visual UI

---

### 1.2 MCP Server Live Connection
**Problem**: MCP server returns simulated responses, no actual editor connection

**Solution**:
1. Implement IPC mechanism (crossbeam-channel or tokio mpsc)
2. Create message protocol between editor and MCP server
3. Thread-safe command queue in EditorApp
4. Execute MCP commands on editor's event loop

**Files to modify**:
- `/home/k/game-engine/crates/engine-editor/src/ipc.rs` (new)
- `/home/k/game-engine/crates/engine-mcp-server/src/main.rs`

**Impact**: HIGH - Enables real Claude Code control of engine

---

### 1.3 Full Asset Hot-Reload
**Problem**: Only scripts hot-reload; textures/models detected but not reloaded

**Solution**:
1. Implement texture reload in AssetManager
2. GPU resource invalidation and re-upload
3. Model reload with mesh cache clearing
4. Material rebinding

**Files to modify**:
- `/home/k/game-engine/crates/engine-assets/src/manager.rs`
- `/home/k/game-engine/crates/engine-render/src/renderer.rs`

**Impact**: MEDIUM - Improves iteration speed for artists

---

## Priority 2: Essential Features (High Value)

### 2.1 Audio System
**Why**: Core feature missing from most games

**Implementation**:
- Use `rodio` or `kira` crate for audio playback
- AudioSource component (3D spatial audio)
- AudioListener component (attached to camera)
- Audio asset loading (MP3, OGG, WAV)
- Audio mixing and volume control

**New crate**: `engine-audio`

**API Example**:
```rust
// In Rhai script
audio.play_sound("sounds/jump.ogg");
audio.play_music("music/level1.mp3", loop=true);
entity.play_3d_sound("sounds/footstep.wav", volume=0.8);
```

**Impact**: HIGH - Required for most games

---

### 2.2 Shadow Mapping
**Why**: Dramatically improves visual quality

**Implementation**:
- Shadow map render pass (depth-only)
- Directional light shadows (cascaded shadow maps)
- Point light shadows (cubemap shadows)
- Percentage-Closer Filtering (PCF) for soft shadows

**Files to modify**:
- `/home/k/game-engine/crates/engine-render/src/renderer.rs`
- `/home/k/game-engine/crates/engine-render/src/shaders/pbr_advanced.wgsl`

**Complexity**: MEDIUM

**Impact**: HIGH - Professional visual quality

---

### 2.3 Particle System
**Why**: Essential for VFX (explosions, fire, smoke, magic)

**Implementation**:
- CPU-based particle simulation (GPU later)
- Particle emitters (point, cone, sphere)
- Particle properties (lifetime, velocity, color gradient, size curve)
- Instanced rendering for performance

**New crate**: `engine-particles`

**API Example**:
```rust
// In Rhai script
let emitter = particle_emitter("fire");
emitter.set_rate(50); // particles per second
emitter.set_lifetime(2.0);
emitter.set_color_gradient([color(1,0,0), color(1,1,0)]);
```

**Impact**: HIGH - Essential for polished games

---

### 2.4 Skybox & Environment Maps
**Why**: Improves scene atmosphere and realistic reflections

**Implementation**:
- Cubemap loading (6-sided texture)
- Skybox shader (rendered as background)
- Environment mapping for reflections
- Image-based lighting (IBL) for ambient

**Files to modify**:
- `/home/k/game-engine/crates/engine-render/src/renderer.rs`
- New shader: `skybox.wgsl`

**Complexity**: LOW-MEDIUM

**Impact**: MEDIUM - Better visual atmosphere

---

### 2.5 Post-Processing Pipeline
**Why**: Modern games require effects like bloom, SSAO, color grading

**Implementation**:
1. **Framebuffer abstraction** - Render to texture
2. **Bloom** - Extract bright pixels, blur, composite
3. **SSAO** - Screen-space ambient occlusion for depth
4. **Tone mapping** - HDR to LDR (already have Reinhard, add ACES)
5. **Color grading** - LUT-based color correction
6. **FXAA/TAA** - Anti-aliasing

**Files to modify**:
- `/home/k/game-engine/crates/engine-render/src/post_processing.rs` (new)
- New shaders for each effect

**Complexity**: MEDIUM-HIGH

**Impact**: HIGH - AAA-level visuals

---

## Priority 3: Advanced Physics (Medium Priority)

### 3.1 Physics Joints & Constraints
**Why**: Enables doors, ropes, chains, vehicles

**Implementation**:
- Fixed joint (weld objects together)
- Hinge joint (doors, wheels)
- Ball-socket joint (ragdolls)
- Prismatic joint (sliding doors)
- Spring/damper (suspension)

**Files to modify**:
- `/home/k/game-engine/crates/engine-physics/src/joints.rs` (new)

**Rapier support**: Built-in, just need to expose API

**Impact**: MEDIUM - Enables physics-based gameplay

---

### 3.2 Ragdoll Physics
**Why**: Realistic character death/reactions

**Implementation**:
- Skeleton-to-rigidbody mapping
- Joint chain creation from bone hierarchy
- Animated-to-ragdoll transition
- Blended animation-ragdoll hybrid

**Complexity**: HIGH

**Impact**: MEDIUM - Polished character interactions

---

### 3.3 Raycast Improvements
**Current limitations**: No entity lookup, placeholder normals

**Improvements**:
1. Fix entity lookup via body_to_entity map (make public or add accessor)
2. Calculate proper surface normals from collider shapes
3. Add layer filtering (collision layers/masks)
4. Add raycast visualization in editor

**Files to modify**:
- `/home/k/game-engine/crates/engine-physics/src/raycast.rs`
- `/home/k/game-engine/crates/engine-physics/src/world.rs`

**Complexity**: LOW

**Impact**: MEDIUM - Better gameplay mechanics

---

## Priority 4: Editor Improvements (Quality of Life)

### 4.1 Visual Scene Editor
**Current**: Code-only entity creation

**Add**:
- Click viewport to place entities
- Gizmos for move/rotate/scale (3D manipulators)
- Drag entities in hierarchy to reparent
- Duplicate entities (Ctrl+D)
- Undo/redo system

**Complexity**: HIGH

**Impact**: HIGH - Massive productivity boost

---

### 4.2 Asset Browser Panel
**Why**: Currently no visual asset management

**Implementation**:
- Thumbnail previews for models/textures
- Drag-and-drop from browser to scene
- Asset import settings
- Asset search/filter

**Complexity**: MEDIUM

**Impact**: MEDIUM - Better artist workflow

---

### 4.3 Visual Profiler
**Why**: Performance debugging essential for optimization

**Implementation**:
- Frame time graph
- CPU/GPU profiling
- Memory usage tracking
- Draw call counter
- Physics step timing

**Use**: `puffin` crate for profiling

**Complexity**: MEDIUM

**Impact**: MEDIUM - Essential for optimization

---

### 4.4 Play Mode
**Current**: Always running simulation

**Add**:
- Play/Pause/Step buttons
- Separate edit mode and play mode
- Save state before play, restore on stop
- Runtime script debugging

**Complexity**: MEDIUM

**Impact**: HIGH - Critical editor feature

---

## Priority 5: Performance Optimizations

### 5.1 Frustum Culling
**Why**: Don't render objects outside camera view

**Implementation**:
- Calculate camera frustum planes
- Test entity bounding boxes against frustum
- Skip render for culled objects

**Expected speedup**: 2-5x in complex scenes

**Complexity**: LOW

**Impact**: HIGH - Free performance

---

### 5.2 Level of Detail (LOD)
**Why**: Render distant objects with fewer polygons

**Implementation**:
- Multiple mesh versions per model (LOD0, LOD1, LOD2)
- Distance-based LOD switching
- Smooth transitions

**Complexity**: MEDIUM

**Impact**: MEDIUM - Better performance in large worlds

---

### 5.3 Occlusion Culling
**Why**: Don't render objects behind other objects

**Implementation**:
- Hi-Z buffer approach
- Query-based culling
- Hardware occlusion queries

**Complexity**: HIGH

**Impact**: MEDIUM - Large scene performance

---

### 5.4 Multithreading
**Current**: Single-threaded rendering and physics

**Add**:
- Parallel system updates
- Multi-threaded asset loading
- Job system for scripts

**Consider**: Migrate to ECS (Bevy or custom) for data-oriented parallelism

**Complexity**: VERY HIGH

**Impact**: HIGH - Multi-core utilization

---

## Priority 6: Production Features

### 6.1 Build System
**Why**: Package games as standalone executables

**Implementation**:
- Asset packing (bundle all assets into single file)
- Release build optimization
- Platform-specific packaging (Windows .exe, Linux AppImage, macOS .app)
- Icon and metadata

**Complexity**: MEDIUM

**Impact**: CRITICAL for distribution

---

### 6.2 Input System Improvements
**Current**: Hardcoded input checks

**Add**:
- Input mapping (rebindable controls)
- Gamepad support (via gilrs crate)
- Touch support (mobile future-proofing)
- Input buffering

**Complexity**: MEDIUM

**Impact**: MEDIUM - Better player experience

---

### 6.3 UI Framework
**Current**: egui (editor-focused, not game UI)

**Add game-specific UI**:
- Option A: Use `bevy_ui` (immediate mode)
- Option B: Use `egui` styled for games
- Option C: Custom UI system with layouts

**Features needed**:
- Health bars
- Menus
- Dialogs
- HUD elements

**Complexity**: HIGH

**Impact**: HIGH - Required for game UI

---

## Priority 7: Advanced Features (Nice to Have)

### 7.1 Terrain System
- Heightmap-based terrain
- Terrain sculpting tools
- Texture splatting (multi-layer materials)
- Grass/foliage instancing

**Complexity**: HIGH

**Impact**: MEDIUM - For open-world games

---

### 7.2 Animation System
**Current**: No skeletal animation

**Add**:
- Skeletal animation playback
- Animation blending
- State machines
- IK (Inverse Kinematics)

**Complexity**: VERY HIGH

**Impact**: CRITICAL for character-based games

---

### 7.3 Networking
- Client-server architecture
- Entity replication
- Lag compensation
- Interpolation/prediction

**Complexity**: VERY HIGH

**Impact**: CRITICAL for multiplayer games

---

### 7.4 Scripting Language Improvements
**Current**: Rhai is good but limited

**Add**:
- Visual scripting (node-based)
- Better IDE integration (LSP server)
- Debugger (breakpoints, step-through)
- Performance profiling

**Complexity**: HIGH

**Impact**: MEDIUM - Better scripting DX

---

## Recommended Implementation Order

### Phase 9: Critical Fixes (1-2 weeks)
1. Fix egui rendering
2. Implement MCP IPC
3. Complete asset hot-reload
4. Fix raycast entity lookup

**Outcome**: Engine fully functional with no known blockers

---

### Phase 10: Core Features (3-4 weeks)
1. Audio system
2. Shadow mapping
3. Particle system
4. Post-processing (bloom, SSAO)

**Outcome**: Feature-complete for most indie games

---

### Phase 11: Editor Polish (2-3 weeks)
1. Visual scene editor with gizmos
2. Asset browser
3. Play/Pause mode
4. Visual profiler

**Outcome**: Professional editor experience

---

### Phase 12: Optimization (2 weeks)
1. Frustum culling
2. LOD system
3. Performance profiling

**Outcome**: Smooth performance in complex scenes

---

### Phase 13: Production (1-2 weeks)
1. Build system
2. Input mapping
3. Game UI framework

**Outcome**: Ship-ready engine

---

## Long-Term Vision (6+ months)

### Becoming Production-Ready
- Animation system
- Terrain system
- Networking (multiplayer)
- VR support
- Mobile platform support (via wgpu WebGPU)

### Becoming AAA-Quality
- Deferred rendering pipeline
- Advanced material system (shader graph)
- Global illumination (light probes, reflections)
- Advanced AI (navmesh, behavior trees)

---

## Community & Ecosystem

### Documentation
- Video tutorials
- Example projects (FPS, platformer, puzzle)
- API documentation (rustdoc)
- Wiki

### Tooling
- Asset converter tools
- Model importer improvements
- Texture packing
- Plugin system for extensions

### Distribution
- Publish crates to crates.io
- GitHub releases
- Website and landing page
- Community Discord/forum

---

## Metrics for Success

**Indie Game Ready**:
- ✅ Can build a simple 3D game in <1 week
- ✅ Hot-reload iteration time <1 second
- ✅ 60 FPS in scenes with 1000+ objects
- ✅ Full editor with visual tools

**Production Ready**:
- Can ship a commercial game
- Supports all major platforms
- Performance comparable to Unity/Godot
- Active community and documentation

---

## Next Steps

**Immediate**:
1. Decide on egui fix approach (downgrade wgpu vs wait for egui-wgpu 0.31)
2. Implement MCP IPC for real Claude Code control
3. Fix raycast entity lookup

**Short-term**:
4. Add audio system (highest ROI feature)
5. Implement shadow mapping (visual quality jump)

**Long-term**:
6. Visual scene editor (transforms editor UX)
7. Animation system (unlocks character games)
