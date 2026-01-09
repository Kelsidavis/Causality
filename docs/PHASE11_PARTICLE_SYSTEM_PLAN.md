# Phase 11: GPU Particle System Implementation Plan

## Overview

Implement a GPU-accelerated particle system with compute shaders for thousands of particles. This will enable visual effects like fire, smoke, explosions, magic spells, and environmental effects.

**Status:** Planning Phase
**Estimated Complexity:** High
**Estimated Impact:** Very High (visual quality)
**Dependencies:** Phase 9 (advanced rendering pipeline)

## Goals

### Primary Goals
1. ✅ GPU-accelerated particle simulation using compute shaders
2. ✅ Support for 10,000+ particles at 60 FPS
3. ✅ Multiple emitter shapes (point, sphere, cone, box)
4. ✅ Particle properties (lifetime, velocity, size, color over time)
5. ✅ Texture-based particles with alpha blending
6. ✅ Integration with scene component system

### Secondary Goals
1. Physics interaction (collision with world)
2. LOD system for distant particle emitters
3. Particle pooling and recycling
4. Multiple blending modes (additive, alpha, multiplicative)

### Stretch Goals
1. Soft particles (depth-aware blending)
2. Particle attractors/repellers
3. Particle trails
4. GPU-based collision detection

## Architecture

### Crate Structure

```
crates/
├── engine-particles/          # New crate
│   ├── src/
│   │   ├── lib.rs
│   │   ├── particle.rs        # Particle data structure
│   │   ├── emitter.rs         # Emitter shapes and properties
│   │   ├── system.rs          # Particle system management
│   │   └── compute.rs         # GPU compute pipeline
└── engine-render/
    └── src/
        └── particle_renderer.rs  # Particle rendering pipeline
```

### Data Flow

```
ParticleEmitter (Component)
    ↓
ParticleSystem (CPU)
    ↓ Spawn particles
GPU Compute Shader (Update)
    ↓ Simulate physics
GPU Buffer (Read/Write)
    ↓
Instanced Rendering
    ↓
Screen Output
```

## Technical Design

### 1. Particle Structure (GPU-compatible)

```rust
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct GpuParticle {
    position: [f32; 3],
    _padding1: f32,
    velocity: [f32; 3],
    _padding2: f32,
    color: [f32; 4],          // RGBA with alpha
    size: f32,
    lifetime: f32,             // Current lifetime
    max_lifetime: f32,         // Total lifetime
    rotation: f32,             // Rotation angle
}
```

**Total size:** 64 bytes (GPU-friendly alignment)

### 2. Emitter Types

```rust
pub enum EmitterShape {
    Point,                     // Single point emission
    Sphere { radius: f32 },    // Spherical emission
    Cone { angle: f32, radius: f32 },  // Cone-shaped emission
    Box { size: Vec3 },        // Box-shaped emission
    Circle { radius: f32 },    // Flat circle emission
}

pub struct EmitterProperties {
    pub shape: EmitterShape,
    pub rate: f32,             // Particles per second
    pub initial_velocity: Vec3,
    pub velocity_randomness: f32,
    pub lifetime: f32,
    pub lifetime_randomness: f32,
    pub initial_size: f32,
    pub size_over_lifetime: Vec<f32>,  // Curve
    pub initial_color: [f32; 4],
    pub color_over_lifetime: Vec<[f32; 4]>,  // Gradient
}
```

### 3. Compute Shader Pipeline

**Compute Shader:** `particle_update.wgsl`

```wgsl
struct Particle {
    position: vec3<f32>,
    velocity: vec3<f32>,
    color: vec4<f32>,
    size: f32,
    lifetime: f32,
    max_lifetime: f32,
    rotation: f32,
}

@group(0) @binding(0) var<storage, read_write> particles: array<Particle>;
@group(0) @binding(1) var<uniform> time: f32;
@group(0) @binding(2) var<uniform> delta_time: f32;
@group(0) @binding(3) var<uniform> gravity: vec3<f32>;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;

    if (index >= arrayLength(&particles)) {
        return;
    }

    var particle = particles[index];

    // Update lifetime
    particle.lifetime += delta_time;

    // Kill if expired
    if (particle.lifetime >= particle.max_lifetime) {
        particle.position = vec3<f32>(0.0, -9999.0, 0.0);  // Hide offscreen
        return;
    }

    // Update velocity (gravity)
    particle.velocity += gravity * delta_time;

    // Update position
    particle.position += particle.velocity * delta_time;

    // Update size over lifetime
    let life_ratio = particle.lifetime / particle.max_lifetime;
    particle.size = mix(1.0, 0.0, life_ratio);  // Shrink over time

    // Update color/alpha over lifetime
    particle.color.a = 1.0 - life_ratio;  // Fade out

    // Write back
    particles[index] = particle;
}
```

### 4. Instanced Rendering

**Vertex Shader:** Billboard quads facing camera

```wgsl
struct ParticleInstance {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
    @location(2) size: f32,
    @location(3) rotation: f32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
}

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32,
    instance: ParticleInstance,
) -> VertexOutput {
    var output: VertexOutput;

    // Quad vertices (camera-facing billboard)
    let quad_positions = array<vec2<f32>, 4>(
        vec2<f32>(-0.5, -0.5),
        vec2<f32>( 0.5, -0.5),
        vec2<f32>(-0.5,  0.5),
        vec2<f32>( 0.5,  0.5),
    );

    let quad_pos = quad_positions[vertex_index];

    // Billboard facing camera
    let world_pos = instance.position +
                    camera_right * quad_pos.x * instance.size +
                    camera_up * quad_pos.y * instance.size;

    output.clip_position = view_proj * vec4<f32>(world_pos, 1.0);
    output.uv = quad_pos + 0.5;  // [0, 1] range
    output.color = instance.color;

    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Sample particle texture
    let tex_color = textureSample(particle_texture, particle_sampler, input.uv);

    // Apply color tint and alpha
    return tex_color * input.color;
}
```

### 5. Component Integration

```rust
// Component for scene entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticleEmitter {
    pub enabled: bool,
    pub max_particles: u32,
    pub emitter_properties: EmitterProperties,
    pub texture_path: Option<String>,
    pub blend_mode: BlendMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BlendMode {
    Alpha,      // Standard alpha blending
    Additive,   // Additive (fire, sparks)
    Multiply,   // Multiplicative (smoke)
}
```

## Implementation Phases

### Phase 1: Core Infrastructure (Week 1)

**Goal:** Basic particle data structures and GPU buffers

**Tasks:**
1. Create `engine-particles` crate
2. Define `GpuParticle` struct with proper alignment
3. Create particle buffer management (double-buffered)
4. Implement basic ParticleSystem CPU structure
5. Add ParticleEmitter component to engine-scene

**Deliverables:**
- `crates/engine-particles/src/particle.rs`
- `crates/engine-particles/src/emitter.rs`
- `crates/engine-particles/src/system.rs`
- Particle buffers created on GPU

### Phase 2: Compute Pipeline (Week 1)

**Goal:** GPU-accelerated particle simulation

**Tasks:**
1. Create compute shader (`particle_update.wgsl`)
2. Implement compute pipeline in `compute.rs`
3. Set up storage buffers for read/write
4. Implement dispatch logic (workgroups)
5. Basic particle spawning (emit from point)

**Deliverables:**
- `crates/engine-render/src/shaders/particle_update.wgsl`
- `crates/engine-particles/src/compute.rs`
- Working compute shader that updates particles

### Phase 3: Rendering Pipeline (Week 2)

**Goal:** Render particles as instanced billboards

**Tasks:**
1. Create particle vertex/fragment shaders
2. Implement instanced rendering
3. Billboard orientation (face camera)
4. Texture support with alpha blending
5. Multiple blend modes (additive, alpha, multiply)

**Deliverables:**
- `crates/engine-render/src/shaders/particle.wgsl`
- `crates/engine-render/src/particle_renderer.rs`
- Particles visible on screen

### Phase 4: Emitter Shapes (Week 2)

**Goal:** Support multiple emitter shapes

**Tasks:**
1. Implement sphere emitter
2. Implement cone emitter
3. Implement box emitter
4. Implement circle emitter
5. Randomization within shapes

**Deliverables:**
- All emitter shapes working
- Proper initial velocity distribution

### Phase 5: Property Curves (Week 3)

**Goal:** Particle properties change over lifetime

**Tasks:**
1. Size over lifetime curves
2. Color over lifetime gradients
3. Velocity damping
4. Rotation over time
5. Alpha fadeout

**Deliverables:**
- Particle properties animate smoothly
- Configurable curves in EmitterProperties

### Phase 6: Integration & Polish (Week 3)

**Goal:** Full scene integration and examples

**Tasks:**
1. Integrate with editor
2. Create example particle effects:
   - Fire
   - Smoke
   - Explosion
   - Magic sparkles
   - Rain
3. Performance optimization
4. LOD system (reduce particles at distance)

**Deliverables:**
- 5+ example particle systems
- Scene files with particle emitters
- Performance: 10,000 particles @ 60 FPS

### Phase 7: Documentation (Week 4)

**Goal:** Complete documentation and examples

**Tasks:**
1. Write PARTICLES.md guide
2. Document EmitterProperties
3. Create example scripts
4. Performance profiling
5. Update FEATURES.md

**Deliverables:**
- `docs/PARTICLES.md`
- Example particle effect files
- Performance benchmarks

## File Structure

### New Files to Create (~12 files)

```
crates/engine-particles/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── particle.rs           # GpuParticle struct
    ├── emitter.rs            # Emitter shapes and properties
    ├── system.rs             # ParticleSystem management
    └── compute.rs            # Compute pipeline

crates/engine-render/src/
├── particle_renderer.rs      # Rendering pipeline
└── shaders/
    ├── particle.wgsl         # Vertex/Fragment shaders
    └── particle_update.wgsl  # Compute shader

assets/
├── particles/
│   ├── fire.particle
│   ├── smoke.particle
│   ├── explosion.particle
│   ├── sparkles.particle
│   └── rain.particle
└── textures/particles/
    ├── particle_default.png
    ├── fire.png
    ├── smoke.png
    └── sparkle.png

docs/
└── PARTICLES.md
```

### Files to Modify (~8 files)

1. `Cargo.toml` - Add engine-particles crate
2. `crates/engine-scene/src/components.rs` - Add ParticleEmitter
3. `crates/engine-editor/src/main.rs` - Integrate particle system
4. `crates/engine-render/src/lib.rs` - Export particle_renderer
5. `crates/engine-render/src/renderer.rs` - Add particle rendering pass
6. `FEATURES.md` - Document particle system
7. `docs/QUICK_REFERENCE.md` - Add particle examples
8. `Cargo.lock` - Updated dependencies

## Performance Targets

### Target Performance

- **10,000 particles** @ 60 FPS on mid-range GPU
- **50,000 particles** @ 30 FPS on high-end GPU
- **Compute overhead:** <2ms for 10k particles
- **Render overhead:** <3ms for 10k particles
- **Memory:** ~640 KB for 10k particles (64 bytes each)

### Optimization Strategies

1. **Double-buffered compute** - Ping-pong between buffers
2. **Particle pooling** - Reuse dead particles
3. **LOD** - Reduce particle count at distance
4. **Frustum culling** - Don't update offscreen emitters
5. **Workgroup size** - Optimize for GPU (256 threads)

## Technical Challenges

### Challenge 1: GPU Synchronization

**Problem:** Compute shader writes, vertex shader reads - need synchronization

**Solution:**
- Use double-buffering (ping-pong)
- Explicit barriers between compute and render
- wgpu handles synchronization automatically

### Challenge 2: Particle Spawning

**Problem:** Spawning new particles from CPU while GPU updates

**Solution:**
- Reserve spawn buffer in GPU memory
- CPU writes spawn commands
- Compute shader consumes spawn queue and activates particles

### Challenge 3: Billboard Orientation

**Problem:** Particles must face camera at all angles

**Solution:**
- Pass camera right/up vectors to vertex shader
- Construct billboard quad in camera space
- Or use geometry shader (if available)

### Challenge 4: Sorting for Transparency

**Problem:** Alpha-blended particles need back-to-front sorting

**Solution:**
- Use additive blending (no sorting needed)
- Or implement GPU sorting (radix sort)
- Or accept visual artifacts (most games do this)

## Testing Plan

### Unit Tests
- Emitter shape calculations
- Particle lifetime updates
- Buffer management

### Integration Tests
- Compute shader execution
- Instanced rendering
- Multiple emitters simultaneously

### Visual Tests
- Create test scene with all emitter types
- Verify particle behavior (gravity, fade, etc.)
- Performance profiling with 10k+ particles

### Benchmark Tests
- Particle spawn rate
- Update performance (compute)
- Render performance (instancing)

## Success Criteria

Phase 11 is complete when:

1. ✅ 10,000 particles render at 60 FPS
2. ✅ All emitter shapes working (point, sphere, cone, box)
3. ✅ Particle properties animate over lifetime
4. ✅ Additive and alpha blending modes work
5. ✅ At least 5 example particle effects created
6. ✅ Full documentation in PARTICLES.md
7. ✅ Integrated with scene system and editor
8. ✅ No visual glitches or crashes

## Example Particle Effects

### Fire Effect
```ron
(
    type: "ParticleEmitter",
    enabled: true,
    max_particles: 1000,
    emitter_properties: (
        shape: Cone(angle: 0.3, radius: 0.5),
        rate: 100.0,
        initial_velocity: (0.0, 2.0, 0.0),
        velocity_randomness: 0.3,
        lifetime: 1.5,
        initial_size: 0.5,
        size_over_lifetime: [1.0, 0.8, 0.3, 0.0],
        initial_color: [1.0, 0.8, 0.2, 1.0],
        color_over_lifetime: [
            [1.0, 0.8, 0.2, 1.0],
            [1.0, 0.4, 0.0, 0.8],
            [0.3, 0.1, 0.0, 0.3],
            [0.0, 0.0, 0.0, 0.0],
        ],
    ),
    texture_path: Some("textures/particles/fire.png"),
    blend_mode: Additive,
)
```

### Smoke Effect
```ron
(
    type: "ParticleEmitter",
    enabled: true,
    max_particles: 500,
    emitter_properties: (
        shape: Sphere(radius: 0.3),
        rate: 30.0,
        initial_velocity: (0.0, 1.0, 0.0),
        velocity_randomness: 0.5,
        lifetime: 3.0,
        initial_size: 0.3,
        size_over_lifetime: [0.3, 0.8, 1.2, 1.5],
        initial_color: [0.5, 0.5, 0.5, 0.8],
        color_over_lifetime: [
            [0.5, 0.5, 0.5, 0.8],
            [0.4, 0.4, 0.4, 0.5],
            [0.3, 0.3, 0.3, 0.2],
            [0.2, 0.2, 0.2, 0.0],
        ],
    ),
    texture_path: Some("textures/particles/smoke.png"),
    blend_mode: Alpha,
)
```

## Resources Needed

### Assets
- Particle texture pack (fire, smoke, sparkles, etc.)
- Example particle configurations

### Libraries
- `bytemuck` - Already used for GPU data
- `wgpu` - Already used for rendering
- No new dependencies needed!

### Learning Resources
- [GPU Gems 3: Chapter 23 - Particle Simulation](https://developer.nvidia.com/gpugems/gpugems3/part-iv-image-effects/chapter-23-high-speed-high-quality-rendering-particles)
- [Real-Time Particle Systems (Valve)](https://www.valvesoftware.com/en/publications)
- [wgpu Compute Shader Tutorial](https://sotrh.github.io/learn-wgpu/compute/)

## Risk Assessment

### Low Risk
- Basic particle spawning and rendering
- Point emitter implementation
- Documentation

### Medium Risk
- Compute shader complexity
- GPU synchronization
- Performance at high particle counts

### High Risk
- Particle sorting for transparency
- Physics collision with particles
- Soft particles (depth-aware)

**Mitigation:** Start with low-risk features, add high-risk features as stretch goals.

## Next Steps

1. **Review this plan** - Ensure approach is sound
2. **Create engine-particles crate** - Set up project structure
3. **Implement Phase 1** - Core infrastructure
4. **Test early and often** - Validate GPU pipeline works

---

**Status:** 🎯 Ready for Phase 11 implementation!
**Estimated Duration:** 3-4 weeks
**Expected LOC:** ~2,000 lines
**Expected Visual Impact:** ⭐⭐⭐⭐⭐ Excellent!

