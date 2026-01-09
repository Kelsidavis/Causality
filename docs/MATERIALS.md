# Material System Guide

## Overview

The Causality Engine uses a PBR (Physically Based Rendering) material system with support for multiple texture maps and advanced rendering features including normal mapping, metallic-roughness workflow, and bloom post-processing.

## Material File Format

Materials are defined in YAML files (`.mat` extension) located in the `assets/materials/` directory.

### Basic Structure

```yaml
name: "Material Name"

# Texture paths (relative to assets/ directory)
albedo_texture: "textures/albedo.png"
normal_texture: "textures/normal.png"
metallic_roughness_texture: "textures/metallic_roughness.png"
ao_texture: "textures/ao.png"

# PBR parameters
base_color: [1.0, 1.0, 1.0, 1.0]  # RGBA (0-1)
metallic: 0.0                      # 0.0 = dielectric, 1.0 = metal
roughness: 0.5                     # 0.0 = smooth, 1.0 = rough
ao_factor: 1.0                     # Ambient occlusion multiplier

# Emission (for bloom effects)
emissive_color: [0.0, 0.0, 0.0]   # RGB
emissive_strength: 0.0             # Multiplier for bloom

# Rendering options
alpha_mode: Opaque                 # Opaque, Blend, or Mask
alpha_cutoff: 0.5                  # For alpha masking
double_sided: false                # Render both sides
```

## Texture Maps

### Albedo Texture
- Base color of the material
- RGB channels contain color information
- Should be in sRGB color space

### Normal Map
- Tangent-space normal map
- RGB channels encode surface normals
- Required format: OpenGL-style (Y+ up)
- Used for surface detail without additional geometry

### Metallic-Roughness Texture
- **Blue channel**: Metallic values (0 = dielectric, 1 = metal)
- **Green channel**: Roughness values (0 = smooth, 1 = rough)
- Follows glTF 2.0 specification

### Ambient Occlusion (AO) Texture
- Red channel contains AO values
- Darkens occluded areas for more realistic shading

## PBR Parameters

### Metallic
- **0.0**: Non-metallic materials (wood, stone, plastic)
- **1.0**: Metals (iron, gold, chrome)
- Values between create blended effects

### Roughness
- **0.0**: Perfect mirror (chrome, polished metal)
- **0.5**: Semi-glossy (painted surfaces)
- **1.0**: Completely diffuse (rough stone, fabric)

### Base Color
- Multiplied with albedo texture
- Use `[1.0, 1.0, 1.0, 1.0]` to show texture as-is
- Can tint textures by adjusting RGB values

## Example Materials

### Polished Metal
```yaml
name: "Chrome"
albedo_texture: null
normal_texture: null
metallic_roughness_texture: null
ao_texture: null

base_color: [0.95, 0.95, 0.95, 1.0]
metallic: 1.0
roughness: 0.05
ao_factor: 1.0

emissive_color: [0.0, 0.0, 0.0]
emissive_strength: 0.0

alpha_mode: Opaque
double_sided: false
```

### Rough Stone
```yaml
name: "Stone"
albedo_texture: "textures/stone_albedo.png"
normal_texture: "textures/stone_normal.png"
metallic_roughness_texture: null
ao_texture: null

base_color: [1.0, 1.0, 1.0, 1.0]
metallic: 0.0
roughness: 0.9
ao_factor: 1.0

emissive_color: [0.0, 0.0, 0.0]
emissive_strength: 0.0

alpha_mode: Opaque
double_sided: false
```

### Emissive Glow (for Bloom)
```yaml
name: "Neon Glow"
albedo_texture: null
normal_texture: null
metallic_roughness_texture: null
ao_texture: null

base_color: [0.1, 0.1, 0.2, 1.0]
metallic: 0.0
roughness: 0.5
ao_factor: 1.0

emissive_color: [1.0, 0.4, 0.2]    # Orange glow
emissive_strength: 3.0              # Bright for bloom

alpha_mode: Opaque
double_sided: false
```

## Using Materials in Scenes

In `.ron` scene files, assign materials to entities:

```ron
(
    type: "MeshRenderer",
    mesh_path: "stone_cube",
    material_path: Some("materials/stone_bricks.mat"),
)
```

If no material is specified, the default white material is used:
```ron
material_path: None,  // Uses materials/default.mat
```

## Technical Details

### Shader Architecture
- **Cook-Torrance BRDF**: Industry-standard PBR lighting model
- **GGX Distribution**: Microfacet normal distribution
- **Smith Geometry**: Geometric shadowing/masking
- **Fresnel-Schlick**: Specular reflection approximation

### Bind Group Layout
Materials use bind group 1 with 9 bindings:
- Binding 0: Material uniforms (64 bytes)
- Bindings 1-2: Albedo texture + sampler
- Bindings 3-4: Normal texture + sampler
- Bindings 5-6: Metallic-roughness texture + sampler
- Bindings 7-8: AO texture + sampler

### Normal Mapping
- Requires tangent and bitangent vertex attributes
- Uses mikktspace algorithm for tangent generation
- Automatic TBN matrix construction in vertex shader
- Transforms tangent-space normals to world space

### Bloom Post-Processing
- Objects with `emissive_strength > 1.0` will glow
- Bright pass extracts luminance above threshold
- Gaussian blur creates glow effect
- Additive blending with scene

## Performance Considerations

### Texture Resolution
- Albedo: 1024x1024 to 2048x2048 recommended
- Normal maps: Same as albedo for best detail
- Metallic-roughness: Can be lower (512x512)
- AO: Can be lowest (256x256 to 512x512)

### Vertex Data
- Extended format: 80 bytes per vertex
- Includes tangent (16 bytes) and bitangent (12 bytes)
- LOD system helps with large meshes

### Material Caching
- Materials are loaded once and cached
- Textures are shared between materials
- GPU upload happens on first use

## Common Workflows

### Creating a New Material

1. Create a `.mat` file in `assets/materials/`
2. Define texture paths and PBR parameters
3. Reference in scene file
4. Editor will load automatically

### Testing Materials

Run the PBR showcase scene:
```bash
cargo run --bin editor -- --scene assets/scenes/pbr_showcase.ron
```

### Debugging

- Use `materials/default.mat` as a fallback
- Check console logs for material loading errors
- Verify texture paths are relative to `assets/`
- Ensure texture files exist and are valid images

## Advanced Features

### Normal Mapping
Normal maps add surface detail without geometry. The engine:
1. Loads normal map texture
2. Calculates tangent space (mikktspace)
3. Constructs TBN matrix per-vertex
4. Transforms normals in fragment shader

### Bloom Effects
For glowing objects:
1. Set `emissive_color` to desired glow color
2. Set `emissive_strength` to 2.0 or higher
3. Engine automatically applies bloom post-processing
4. Adjustable bloom intensity and threshold

### Shadow Mapping
All materials automatically receive shadows:
- 2048x2048 shadow map resolution
- PCF (Percentage Closer Filtering) for soft shadows
- Directional light shadow mapping

## Troubleshooting

### Material not loading
- Check file path in console logs
- Verify YAML syntax is valid
- Ensure material file has `.mat` extension

### Textures not showing
- Verify texture paths relative to `assets/`
- Check image format (PNG, JPG supported)
- Look for texture loading errors in console

### Incorrect lighting
- Verify normal map format (OpenGL-style, Y+ up)
- Check metallic/roughness values are in 0-1 range
- Ensure base_color isn't too dark

### No bloom effect
- Increase `emissive_strength` (try 2.5 or higher)
- Set `emissive_color` to bright color
- Check post-processing is enabled

## Future Enhancements

Planned features:
- Parallax occlusion mapping (height maps)
- Clearcoat/sheen layers for complex materials
- Subsurface scattering for skin/wax
- Anisotropic reflections for brushed metal
- Image-based lighting (HDR environment maps)
