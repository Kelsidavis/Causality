# Texture System Documentation

The Causality Engine features a comprehensive texture rendering system that supports loading, managing, and rendering textures on 3D meshes with full GPU acceleration.

## Architecture Overview

The texture system consists of three main components:

1. **GpuTexture** - GPU-side texture representation
2. **TextureManager** - Texture loading and caching
3. **Renderer Integration** - Shader and pipeline configuration

## Components

### GpuTexture (`gpu_texture.rs`)

Represents a texture on the GPU with all necessary resources for rendering.

**Structure:**
```rust
pub struct GpuTexture {
    pub texture: wgpu::Texture,       // GPU texture resource
    pub view: wgpu::TextureView,       // Texture view for sampling
    pub sampler: wgpu::Sampler,        // Sampling configuration
    pub bind_group: wgpu::BindGroup,   // Shader binding
}
```

**Features:**
- Automatic format conversion (RGB8 → RGBA8)
- sRGB color space support
- Linear filtering with repeat wrapping
- Fallback white texture support

**Usage:**
```rust
let gpu_texture = GpuTexture::from_cpu_texture(
    &device,
    &queue,
    &texture,
    &bind_group_layout
);
```

### TextureManager (`texture_manager.rs`)

Manages texture loading, caching, and GPU upload.

**Features:**
- Texture caching by name (prevents duplicate uploads)
- Automatic white texture fallback
- Shared bind group layout
- Handle-based texture access

**Usage:**
```rust
// Create manager
let mut texture_manager = TextureManager::new(&device, &queue);

// Load and upload texture
let texture = Texture::from_file("path/to/texture.png")?;
let handle = texture_manager.upload_texture(
    &device,
    &queue,
    "my_texture".to_string(),
    &texture
);

// Get texture for rendering
let gpu_texture = texture_manager.get_texture(handle).unwrap();
```

### Shader Integration

The PBR shader samples textures in the fragment stage:

```wgsl
// Texture binding (group 1)
@group(1) @binding(0)
var t_texture: texture_2d<f32>;

@group(1) @binding(1)
var t_sampler: sampler;

// Fragment shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Sample texture at UV coordinates
    let tex_color = textureSample(t_texture, t_sampler, in.tex_coord);

    // Combine with vertex color and lighting
    let base_color = tex_color.rgb * in.color;
    return vec4<f32>(base_color * lighting, tex_color.a);
}
```

## Rendering Pipeline

### 1. Initialization

```rust
// Create renderer (creates texture bind group layout)
let renderer = Renderer::new(&instance, &surface, width, height)?;

// Create texture manager with same device
let mut texture_manager = TextureManager::new(&renderer.device, &renderer.queue);
```

### 2. Loading Textures

```rust
// Load from file
let stone_texture = Texture::from_file("textures/stone.png")?;

// Upload to GPU
texture_manager.upload_texture(
    &renderer.device,
    &renderer.queue,
    "stone".to_string(),
    &stone_texture
);
```

### 3. Rendering

```rust
// Get texture bind group
let handle = texture_manager.get_handle("stone").unwrap();
let texture = texture_manager.get_texture(handle).unwrap();

// Render mesh with texture
renderer.render_mesh(
    &mut encoder,
    &view,
    &depth_texture,
    mesh,
    view_proj,
    model,
    &texture.bind_group,  // Texture bind group
    clear
);
```

## Texture Formats

### Supported Input Formats

- **RGBA8** - 8-bit RGBA (preferred)
- **RGB8** - 8-bit RGB (auto-converted to RGBA)
- **R8** - 8-bit grayscale

### GPU Format

All textures are uploaded as:
- **Rgba8UnormSrgb** - 8-bit RGBA with sRGB color space
- sRGB ensures correct color reproduction
- Automatic gamma correction

### Format Conversion

RGB8 textures are automatically converted to RGBA8:

```rust
if texture.format == TextureFormat::Rgb8 {
    let mut rgba_data = Vec::new();
    for chunk in texture.data.chunks(3) {
        rgba_data.push(chunk[0]); // R
        rgba_data.push(chunk[1]); // G
        rgba_data.push(chunk[2]); // B
        rgba_data.push(255);      // A (fully opaque)
    }
}
```

## Sampler Configuration

Default sampler settings:

```rust
wgpu::SamplerDescriptor {
    address_mode_u: wgpu::AddressMode::Repeat,
    address_mode_v: wgpu::AddressMode::Repeat,
    address_mode_w: wgpu::AddressMode::Repeat,
    mag_filter: wgpu::FilterMode::Linear,
    min_filter: wgpu::FilterMode::Linear,
    mipmap_filter: wgpu::FilterMode::Linear,
    // No mipmaps currently
    lod_min_clamp: 0.0,
    lod_max_clamp: 100.0,
    anisotropy_clamp: 1,
}
```

- **Repeat wrapping** - Textures tile seamlessly
- **Linear filtering** - Smooth interpolation
- **No anisotropic filtering** - Can be added later

## Mesh UV Coordinates

Meshes must have UV coordinates for texture mapping:

```rust
pub struct GpuVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tex_coord: [f32; 2],  // UV coordinates (0.0 to 1.0)
    pub color: [f32; 3],
}
```

### Cube UV Mapping

The default cube mesh has proper UV mapping for each face:

```rust
// Front face (0.0, 0.0) to (1.0, 1.0)
Vertex::new(Vec3::new(-0.5, -0.5, 0.5))
    .with_uv(Vec2::new(0.0, 0.0)),
Vertex::new(Vec3::new(0.5, -0.5, 0.5))
    .with_uv(Vec2::new(1.0, 0.0)),
// ... etc
```

## AI-Generated Textures

### Stable Diffusion Integration

The engine supports loading AI-generated textures from Stable Diffusion/ComfyUI:

```rust
// Generated textures are stored in generated_assets/textures/
let stone_tex = Texture::from_file("generated_assets/textures/stone.png")?;
texture_manager.upload_texture(&device, &queue, "stone".to_string(), &stone_tex);
```

### Workflow

1. **Generate textures** using ComfyUI or Stable Diffusion
2. **Save to** `generated_assets/textures/`
3. **Load in engine** using `Texture::from_file()`
4. **Apply to meshes** during rendering

### Example Prompts

- **Stone walls**: "medieval castle stone wall texture, seamless, high detail, realistic, 4k"
- **Grass terrain**: "grass terrain texture, seamless, natural, realistic, 4k"
- **Water**: "water surface texture, seamless, blue-green, realistic, 4k"

## Performance Considerations

### Texture Caching

Textures are cached by name to prevent duplicate uploads:

```rust
// First call uploads to GPU
let handle1 = texture_manager.upload_texture(device, queue, "stone", &tex);

// Second call returns existing handle (no upload)
let handle2 = texture_manager.upload_texture(device, queue, "stone", &tex);
// handle1 == handle2
```

### Memory Usage

- Textures are stored in GPU VRAM
- CPU texture data can be dropped after upload
- No mipmaps currently (can be added for LOD)

### Best Practices

1. **Use power-of-2 dimensions** for better GPU compatibility (512x512, 1024x1024, 2048x2048)
2. **Compress textures** when possible (RGBA8 is uncompressed)
3. **Share textures** across multiple meshes using handles
4. **Avoid uploading** the same texture multiple times

## Push Constants for Multi-Object Rendering

The texture system works with push constants for efficient multi-object rendering:

```rust
// Each mesh can have different texture but same shader pipeline
for mesh in meshes {
    let texture = select_texture_for_mesh(mesh);

    renderer.render_mesh(
        encoder,
        view,
        depth_texture,
        mesh,
        view_proj,
        mesh.model_matrix,  // Push constant
        &texture.bind_group, // Different texture per mesh
        first_mesh
    );
    first_mesh = false;
}
```

### Pipeline Layout

```rust
bind_group_layouts: &[
    &uniform_bind_group_layout,    // Group 0: View-projection matrix
    &texture_bind_group_layout,    // Group 1: Texture + sampler
],
push_constant_ranges: &[
    PushConstantRange {
        stages: ShaderStages::VERTEX,
        range: 0..64,  // Model matrix (mat4x4 = 64 bytes)
    }
]
```

## Troubleshooting

### Textures appear white

- Check that texture was loaded successfully
- Verify texture handle is valid
- Ensure UV coordinates are in 0.0-1.0 range

### Textures look washed out

- Verify sRGB format is used (`Rgba8UnormSrgb`)
- Check that vertex colors are white (1.0, 1.0, 1.0) to not tint texture

### Device mismatch errors

- Ensure TextureManager and Renderer use the same device
- Create TextureManager AFTER Renderer is initialized

### Format errors

- All textures are converted to RGBA8
- If errors persist, check image file integrity

## Future Enhancements

Planned improvements to the texture system:

- **Mipmapping** - Generate mipmap chains for LOD
- **Compression** - BC7/ASTC texture compression
- **Normal mapping** - Tangent-space normal maps
- **PBR textures** - Metallic, roughness, AO maps
- **Texture arrays** - Store multiple textures in array
- **Anisotropic filtering** - Better quality at oblique angles
- **Async loading** - Load textures on background threads

## See Also

- [README.md](README.md) - Main engine documentation
- [AI_ASSET_WORKFLOW.md](AI_ASSET_WORKFLOW.md) - AI asset generation guide
- `crates/engine-render/src/gpu_texture.rs` - GPU texture implementation
- `crates/engine-render/src/texture_manager.rs` - Texture manager implementation
- `crates/engine-render/src/shaders/pbr.wgsl` - PBR shader with textures
