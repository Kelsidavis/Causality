# Causality Engine - Quick Reference

## Running the Editor

```bash
# Default scene
cargo run --bin editor

# Specific scene
cargo run --bin editor -- --scene assets/scenes/castle.ron

# PBR showcase
cargo run --bin editor -- --scene assets/scenes/pbr_showcase.ron
```

## Creating a Material

1. Create file in `assets/materials/my_material.mat`:

```yaml
name: "My Material"
albedo_texture: "textures/my_texture.png"
normal_texture: null
metallic_roughness_texture: null
ao_texture: null

base_color: [1.0, 1.0, 1.0, 1.0]
metallic: 0.0
roughness: 0.8
ao_factor: 1.0

emissive_color: [0.0, 0.0, 0.0]
emissive_strength: 0.0

alpha_mode: Opaque
double_sided: false
```

2. Use in scene:

```ron
(
    type: "MeshRenderer",
    mesh_path: "stone_cube",
    material_path: Some("materials/my_material.mat"),
)
```

## Material Presets

### Shiny Metal
```yaml
metallic: 1.0
roughness: 0.1
base_color: [0.9, 0.9, 0.9, 1.0]
```

### Rough Stone
```yaml
metallic: 0.0
roughness: 0.9
base_color: [1.0, 1.0, 1.0, 1.0]
```

### Glowing Object (Bloom)
```yaml
emissive_color: [1.0, 0.5, 0.0]
emissive_strength: 2.5
```

### Water
```yaml
metallic: 0.2
roughness: 0.1
base_color: [0.9, 0.95, 1.0, 0.7]
alpha_mode: Blend
```

## Scene File Format

```ron
(
    name: "Scene Name",
    entities: {
        (1): (
            id: (1),
            name: "Object Name",
            transform: (
                position: (0.0, 0.0, 0.0),
                rotation: (0.0, 0.0, 0.0, 1.0),  // Quaternion
                scale: (1.0, 1.0, 1.0),
            ),
            parent: None,
            children: [],
            components: [
                (
                    type: "MeshRenderer",
                    mesh_path: "stone_cube",
                    material_path: Some("materials/stone_bricks.mat"),
                ),
            ],
        ),
    },
)
```

## Available Meshes

- `stone_cube` - Cube with white tint
- `grass_cube` - Cube with white tint
- `water_cube` - Cube with white tint

All meshes have:
- Proper normals
- UV coordinates
- Tangent/bitangent data for normal mapping

## Available Materials

Located in `assets/materials/`:

- `default.mat` - White fallback
- `grass.mat` - Matte grass (roughness 0.95)
- `stone_bricks.mat` - Rough stone (roughness 0.85)
- `water.mat` - Reflective water
- `metal_polished.mat` - Chrome (metallic 1.0, roughness 0.05)
- `emissive_glow.mat` - Orange glow for bloom
- `debug_normals.mat` - Debug material

## PBR Parameter Guide

### Metallic Values
| Value | Material Type |
|-------|--------------|
| 0.0   | Wood, stone, plastic, fabric |
| 0.3   | Semi-metallic (weathered metal) |
| 0.7   | Tarnished metal |
| 1.0   | Pure metal (iron, gold, chrome) |

### Roughness Values
| Value | Surface Type |
|-------|-------------|
| 0.0   | Perfect mirror |
| 0.2   | Polished metal, wet surface |
| 0.5   | Painted surface |
| 0.8   | Rough stone, concrete |
| 1.0   | Completely matte |

### Emissive Strength (for Bloom)
| Value | Effect |
|-------|--------|
| 0.0   | No glow |
| 1.0   | Subtle glow |
| 2.5   | Moderate glow |
| 5.0   | Very bright glow |

## Camera Controls

- **W/A/S/D**: Move camera
- **Mouse**: Look around
- **Shift**: Move faster
- **Space**: Move up
- **Ctrl**: Move down

## Common Issues

### Material not loading
```
[ERROR] Failed to load material 'materials/foo.mat'
```
→ Check file path and YAML syntax

### Black objects
```
Missing texture or incorrect material parameters
```
→ Verify textures exist, check metallic/roughness values

### No bloom effect
```
emissive_strength too low
```
→ Increase to 2.0 or higher

## Building

```bash
# Full build
cargo build --release

# Editor only (faster)
cargo build --bin editor

# Check for errors
cargo check
```

## Project Structure

```
game-engine/
├── assets/
│   ├── materials/       # Material .mat files
│   ├── textures/        # Texture images
│   ├── scenes/          # Scene .ron files
│   └── music/           # Audio files
├── crates/
│   ├── engine-assets/   # Asset loading
│   ├── engine-render/   # Rendering pipeline
│   ├── engine-scene/    # Scene management
│   └── engine-editor/   # Editor application
└── docs/                # Documentation
```

## Performance Tips

1. **Use LOD**: Large meshes benefit from level-of-detail
2. **Share Textures**: Multiple materials can use same textures
3. **Frustum Culling**: Automatically culls off-screen objects
4. **Material Caching**: Materials loaded once, reused
5. **Texture Resolution**: Use appropriate sizes (1024x1024 typical)

## Debugging

### Enable Rust backtrace
```bash
RUST_BACKTRACE=1 cargo run --bin editor
```

### Check logs
Look for lines starting with:
- `[ERROR]` - Critical errors
- `[WARN]` - Warnings
- `[INFO]` - General information

### Common log messages
```
Loading scene from: assets/scenes/castle.ron
Loaded scene 'Castle' with 15 entities
Successfully reloaded texture: textures/stone.png
```

## Next Steps

1. Read `docs/MATERIALS.md` for detailed material guide
2. Explore `assets/scenes/pbr_showcase.ron` for examples
3. Create custom materials and test them
4. Experiment with different PBR parameters

## Useful Links

- Material Guide: `docs/MATERIALS.md`
- Implementation Report: `docs/PHASE9_IMPLEMENTATION.md`
- Texture Resources: [freepbr.com](https://freepbr.com), [cc0textures.com](https://cc0textures.com)

---

**Tip:** Start with simple materials (no textures) and add complexity gradually. Use the PBR showcase scene to compare different material properties side-by-side.
