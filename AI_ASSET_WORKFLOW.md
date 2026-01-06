# AI Asset Generation Workflow

This guide explains how to generate and use AI-powered assets (textures, materials, and more) with the Causality Engine using Stable Diffusion and ComfyUI.

## Overview

The Causality Engine integrates with Stable Diffusion to generate high-quality textures and assets on-demand. Generated assets are automatically loaded and rendered in the engine.

## Prerequisites

### Required Software

1. **ComfyUI** - Stable Diffusion user interface
   - Download: https://github.com/comfyanonymous/ComfyUI
   - Install in the engine directory: `./ComfyUI/`

2. **Stable Diffusion Model**
   - Recommended: SD 1.5 or SDXL
   - Place models in `ComfyUI/models/checkpoints/`

3. **Python 3.10+** with required dependencies

## Setup

### 1. Install ComfyUI

```bash
cd /path/to/game-engine

# Clone ComfyUI
git clone https://github.com/comfyanonymous/ComfyUI.git

# Install dependencies
cd ComfyUI
pip install -r requirements.txt

# Download SD model (example: SD 1.5)
wget https://huggingface.co/runwayml/stable-diffusion-v1-5/resolve/main/v1-5-pruned.ckpt \
    -O models/checkpoints/sd-v1-5.ckpt
```

### 2. Configure Network Access (Optional)

For network-wide access (multi-machine workflows):

```bash
# Start ComfyUI with network access
python main.py --listen 0.0.0.0 --port 8188
```

Access from other machines: `http://your-ip:8188`

### 3. Start Services

```bash
# Terminal 1: Start ComfyUI
cd ComfyUI
python main.py

# Terminal 2: Start API wrapper (if using)
cd ..
python comfyui-api-wrapper.py

# Terminal 3: Run the engine
cargo run --bin editor --release
```

## Generating Textures

### Using ComfyUI Interface

1. **Open ComfyUI** at http://localhost:8188
2. **Load workflow** or create new
3. **Configure settings**:
   - Resolution: 512x512 or 1024x1024 (power-of-2 recommended)
   - Prompt: Describe the texture
   - Steps: 20-30 for quality
   - CFG Scale: 7-8 for balanced results

4. **Generate** and save to `generated_assets/textures/`

### Example Prompts

#### Stone/Castle Walls
```
Prompt: "medieval castle stone wall texture, seamless tileable,
high detail photorealistic, weathered gray stone blocks,
mortar between stones, 4k texture"

Negative: "people, sky, ground, objects, text, watermark"
```

#### Grass Terrain
```
Prompt: "grass terrain texture seamless tileable, natural green grass,
realistic blades of grass, varied shades of green,
photorealistic 4k texture"

Negative: "flowers, rocks, dirt patches, people, sky"
```

#### Water
```
Prompt: "water surface texture seamless tileable, calm blue-green water,
subtle ripples, realistic water shader, photorealistic 4k texture"

Negative: "waves, foam, reflections, sky, clouds"
```

#### Wood
```
Prompt: "old wood planks texture seamless tileable, weathered brown wood,
wood grain visible, realistic wooden boards, photorealistic 4k"

Negative: "metal, paint, nails, text"
```

### Seamless/Tileable Textures

For seamless textures that tile without visible seams:

**Method 1: Prompt engineering**
- Include "seamless" and "tileable" in prompt
- Emphasize "no borders", "repeating pattern"

**Method 2: Post-processing**
- Use offset filter in image editor
- Blend seams manually
- Use AI-powered seamless tools

**Method 3: ComfyUI nodes**
- Use "Make Seamless" or "Tile Texture" custom nodes
- Install from ComfyUI Manager

## Integration Workflow

### 1. Generate Assets

```bash
# Generate stone texture via ComfyUI
# Save as: generated_assets/textures/stone_wall.png
```

### 2. Load in Engine

```rust
// In main.rs or asset loading code
use engine_assets::Texture;

// Load AI-generated texture
let stone_texture = Texture::from_file(
    "generated_assets/textures/stone_wall.png"
)?;

// Upload to GPU
texture_manager.upload_texture(
    &device,
    &queue,
    "stone_wall".to_string(),
    &stone_texture
);
```

### 3. Apply to Meshes

```rust
// Get texture for rendering
let texture_name = "stone_wall";
let handle = texture_manager.get_handle(texture_name).unwrap();
let texture = texture_manager.get_texture(handle).unwrap();

// Render mesh with AI-generated texture
renderer.render_mesh(
    encoder,
    view,
    depth_texture,
    mesh,
    view_proj,
    model,
    &texture.bind_group,
    clear
);
```

## Asset Organization

### Directory Structure

```
game-engine/
├── generated_assets/
│   ├── textures/
│   │   ├── stone_wall.png
│   │   ├── grass_field.png
│   │   ├── water_surface.png
│   │   └── wood_planks.png
│   ├── models/         # Future: AI-generated 3D models
│   └── materials/      # Future: Complete material sets
├── ComfyUI/
│   ├── output/         # ComfyUI default output
│   └── models/
└── .gitignore          # Excludes ComfyUI/ and generated_assets/
```

### Naming Conventions

Use descriptive names:
- `stone_wall_medieval.png`
- `grass_field_spring.png`
- `water_calm_lake.png`
- `wood_oak_planks.png`

### File Formats

- **Preferred**: PNG (lossless, supports transparency)
- **Supported**: JPG, WebP, TGA, BMP
- **Resolution**: 512x512, 1024x1024, or 2048x2048 (power-of-2)
- **Color**: RGB or RGBA (auto-converted to RGBA8)

## Advanced Techniques

### ControlNet for Precise Textures

Use ControlNet for controlled generation:

```python
# Example: Generate texture matching a sketch
1. Draw texture layout sketch
2. Use ControlNet (Canny or Depth)
3. Generate with Stable Diffusion
4. Results match sketch structure
```

### Texture Variations

Generate multiple variations:

```python
# Keep same prompt, vary seed
for seed in range(1000, 1010):
    generate_texture(
        prompt="stone wall texture seamless",
        seed=seed,
        output=f"stone_wall_var_{seed}.png"
    )
```

### Material Sets (PBR)

Generate complete material sets for PBR:

**Base Color**
```
Prompt: "stone wall albedo texture seamless, diffuse color only,
no lighting, flat shading, 4k"
```

**Normal Map** (Future support)
```
Prompt: "stone wall normal map texture seamless, purple and blue,
tangent space, detailed surface, 4k"
```

**Roughness** (Future support)
```
Prompt: "stone wall roughness map texture seamless, grayscale,
rough surfaces white, smooth surfaces black, 4k"
```

## API Wrapper Usage

The engine includes a Python API wrapper for programmatic generation:

```python
# comfyui-api-wrapper.py
from comfyui_client import ComfyUIClient

client = ComfyUIClient("http://localhost:8188")

# Queue generation
response = client.generate_texture(
    prompt="medieval stone wall texture seamless",
    negative="people, sky, objects",
    steps=25,
    cfg_scale=7.5,
    seed=42
)

# Save to generated_assets
output_path = f"generated_assets/textures/{response['filename']}"
```

### Batch Generation

Generate multiple assets in batch:

```python
textures = [
    ("stone_wall", "medieval castle stone wall"),
    ("grass_field", "green grass field terrain"),
    ("water", "calm blue water surface"),
    ("wood_planks", "old weathered wood planks")
]

for name, prompt in textures:
    client.generate_texture(
        prompt=f"{prompt} texture seamless 4k",
        output_name=name
    )
```

## Castle Scene Example

The demo castle scene uses AI-generated textures:

```rust
// Load AI-generated textures
let stone = Texture::from_file("generated_assets/textures/generated_85b14ace.png")?;
let grass = Texture::from_file("generated_assets/textures/generated_9c19e917.png")?;
let water = Texture::from_file("generated_assets/textures/generated_ef5a5024.png")?;

// Upload to GPU
texture_manager.upload_texture(&device, &queue, "stone".to_string(), &stone);
texture_manager.upload_texture(&device, &queue, "grass".to_string(), &grass);
texture_manager.upload_texture(&device, &queue, "water".to_string(), &water);

// Create castle structures with textures
// Walls use stone texture
// Terrain uses grass texture
// Moat uses water texture
```

**Results:**
- 4 castle walls with AI-generated stone texture
- Grass terrain with AI-generated grass texture
- Moat with AI-generated water texture
- 19 entities rendered with unique textures

## Performance Tips

### Texture Resolution

- **512x512**: Fast generation, lower detail, good for distant objects
- **1024x1024**: Balanced quality/speed, recommended
- **2048x2048**: High detail, slower generation, for hero assets

### Batch Processing

Generate textures in batches overnight:

```bash
# Create generation script
python batch_generate.py --count 20 --resolution 1024
```

### Caching

The engine caches textures automatically:

```rust
// First load: uploads to GPU
texture_manager.upload_texture(device, queue, "stone", &tex);

// Subsequent loads: uses cached version
texture_manager.upload_texture(device, queue, "stone", &tex); // Fast!
```

## Troubleshooting

### ComfyUI Won't Start

- Check Python version (3.10+ required)
- Install missing dependencies: `pip install -r requirements.txt`
- Verify GPU drivers (CUDA for NVIDIA, ROCm for AMD)

### Out of Memory

- Reduce resolution (1024 → 512)
- Use lower CFG scale
- Clear ComfyUI cache

### Textures Look Bad

- Increase steps (20 → 30)
- Improve prompt quality
- Try different seeds
- Use negative prompts

### Seams Visible

- Add "seamless tileable" to prompt
- Use post-processing to blend edges
- Try different generation settings

## Future Enhancements

Planned AI asset features:

- **3D Model Generation** - AI-generated meshes and objects
- **Material Generation** - Complete PBR material sets
- **Environment Maps** - AI-generated skyboxes and HDRIs
- **Animation** - AI-assisted animation generation
- **Level Design** - Procedural level generation with AI
- **In-Editor Generation** - Generate assets directly in editor
- **Real-time Refinement** - Iterative AI texture refinement

## See Also

- [TEXTURE_SYSTEM.md](TEXTURE_SYSTEM.md) - Texture system documentation
- [README.md](README.md) - Main engine documentation
- ComfyUI Documentation: https://github.com/comfyanonymous/ComfyUI
- Stable Diffusion: https://github.com/CompVis/stable-diffusion

## Community Resources

- **Civitai** - Pre-trained models and LoRAs
- **Hugging Face** - Stable Diffusion models
- **r/StableDiffusion** - Community and tips
- **ComfyUI Workflows** - Pre-made workflows for textures
