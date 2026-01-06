# Stable Diffusion Setup Guide for Causality Engine

This guide explains how to set up and use Stable Diffusion with the Causality Engine for AI-powered asset generation (textures, skyboxes, and other game assets).

## Overview

The Causality Engine integrates with Stable Diffusion in two ways:

1. **Local Service** (Recommended for development) - Run Stable Diffusion on your local machine
2. **Hugging Face API** - Use remote API for cloud-based generation

### Features

- **Texture Generation** - Generate game-ready textures from text descriptions
- **Skybox Generation** - Create 360-degree panoramic skyboxes
- **Quality Levels** - Fast, Standard, High, Best quality options
- **Caching System** - Automatic caching of generated assets to avoid regeneration
- **Prompt Optimization** - Built-in templates and quality-specific prompt enhancement

## Quick Start (Local Setup)

### Option 1: Using AUTOMATIC1111 WebUI (Easiest)

#### 1. Install Dependencies

**On Windows:**
```bash
# Download from: https://github.com/AUTOMATIC1111/stable-diffusion-webui
# Or use git:
git clone https://github.com/AUTOMATIC1111/stable-diffusion-webui
cd stable-diffusion-webui
```

**On macOS:**
```bash
brew install python@3.10
git clone https://github.com/AUTOMATIC1111/stable-diffusion-webui
cd stable-diffusion-webui
```

**On Linux:**
```bash
git clone https://github.com/AUTOMATIC1111/stable-diffusion-webui
cd stable-diffusion-webui
# Install Python 3.10+
```

#### 2. Download Model

The WebUI will automatically download Stable Diffusion 2.1 on first launch, or you can manually place model files in:
```
stable-diffusion-webui/models/Stable-diffusion/
```

**Recommended Models:**
- `sd-v2-1.ckpt` or `sd2-1_768.safetensors` (Stable Diffusion 2.1)
- `sd-xl-1.0.safetensors` (Stable Diffusion XL - highest quality)

Download from:
- [Hugging Face](https://huggingface.co/stabilityai)
- [Model Zoos](https://civitai.com/)

#### 3. Launch WebUI with API

```bash
# Windows
./webui.bat --api

# macOS/Linux
./webui.sh --api
```

Or add to your startup script:
```bash
python launch.py --listen 0.0.0.0 --api --port 7860
```

**Expected Output:**
```
Running on local URL:  http://127.0.0.1:7860
```

#### 4. Verify Installation

Test the API is working:
```bash
curl -X POST http://localhost:7860/api/txt2img \
  -H "Content-Type: application/json" \
  -d '{
    "prompt": "a beautiful landscape",
    "width": 512,
    "height": 512
  }'
```

### Option 2: Using Docker

For containerized deployment:

```bash
# Pull the AUTOMATIC1111 WebUI Docker image
docker pull ghcr.io/automatic1111/stable-diffusion-webui:latest

# Run with API
docker run -d \
  --name stable-diffusion \
  -p 7860:7860 \
  -v /path/to/models:/models \
  ghcr.io/automatic1111/stable-diffusion-webui:latest \
  python launch.py --api --listen 0.0.0.0
```

### Option 3: Using Ollama (Simplified)

For a simplified setup:

```bash
# Install Ollama from https://ollama.ai
# Run Stable Diffusion
ollama pull stable-diffusion

# Start the API (runs on port 5000)
ollama serve
```

## Configuration

### 1. Edit Configuration File

Update `ai-assets-config.toml`:

```toml
[ai_service]
service_type = "local"
local_url = "http://localhost:7860"
local_timeout_seconds = 300

[cache]
cache_dir = "./generated_assets"
enable_cache = true

[generation]
default_quality = "high"
```

### 2. Environment Variables

Alternatively, set environment variables:

```bash
# Local service
export SD_SERVICE_TYPE="local"
export SD_LOCAL_URL="http://localhost:7860"
export SD_TIMEOUT="300"

# Or for Hugging Face
export HF_API_KEY="your_token_here"
export HF_MODEL="stabilityai/stable-diffusion-2-1"

# Cache configuration
export TEXTURE_CACHE_DIR="./generated_assets"
export TEXTURE_QUALITY="high"
```

## Using with Causality Engine

### Via MCP Tools (Recommended)

The Stable Diffusion integration is exposed through MCP tools that Claude Code can call:

```json
{
  "generate_texture": {
    "prompt": "stone brick wall texture",
    "width": 512,
    "height": 512,
    "quality": "high",
    "seed": 42
  }
}
```

```json
{
  "generate_skybox": {
    "prompt": "beautiful sunset sky",
    "quality": "high",
    "seed": 12345
  }
}
```

### Programmatically in Rust

```rust
use engine_ai_assets::{AssetGenerator, AssetCache, TextureGenerationRequest, LocalClient};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create local client
    let client = LocalClient::localhost(7860, 300);

    // Create cache
    let cache = AssetCache::new("./generated_assets")?;

    // Create generator
    let generator = AssetGenerator::new(Box::new(client), cache)?;

    // Generate texture
    let request = TextureGenerationRequest {
        prompt: "wooden plank texture".to_string(),
        negative_prompt: Some("blurry, low quality".to_string()),
        width: 512,
        height: 512,
        steps: 50,
        guidance_scale: 7.5,
        seed: None,
        use_cache: true,
    };

    let asset = generator.generate_texture(&request).await?;
    println!("Generated texture: {}", asset.metadata.file_path);

    Ok(())
}
```

## Prompt Examples

### Texture Generation

```
"high quality wood texture, weathered oak, pbr, 4k, detailed grain patterns"

"seamless stone brick wall, concrete texture, rough surface, tileable"

"metallic steel surface, brushed finish, realistic wear and scratches"

"fabric texture, woven cotton, detailed fibers, high quality"
```

### Skybox Generation

```
"beautiful sunset sky, orange and purple hues, dramatic clouds, 360 panorama"

"serene forest environment, trees, sky, soft lighting, nature scene"

"futuristic city skyline at night, neon lights, cityscape, cyberpunk aesthetic"

"peaceful mountain landscape, snowy peaks, clear sky, scenic vista"
```

### Using Style Templates

The engine provides built-in prompt templates for common asset types:

```rust
use engine_ai_assets::prompt::{templates, styles};

// Create a PBR texture prompt
let prompt = templates::pbr_texture("wood", "weathered");
// Result: "wood material texture, weathered, seamless, 4k, pbr, physically based rendering, detailed surface"

// Create a game-ready texture
let prompt = templates::game_texture("stone", "clean");

// Use style modifiers
let photorealistic = styles::photorealistic();
let pbr_style = styles::pbr_material();
```

## Quality Levels

The engine supports four quality levels:

| Level | Steps | Guidance | Best For | Notes |
|-------|-------|----------|----------|-------|
| `fast` | 20 | 5.0 | Quick previews | Fastest generation, lower quality |
| `standard` | 35 | 7.0 | Development | Good balance of speed and quality |
| `high` | 50 | 7.5 | Production | Recommended default |
| `best` | 75 | 8.5 | Final assets | Highest quality, slower generation |

## Troubleshooting

### Service Not Found

**Error:** "Failed to connect to http://localhost:7860"

**Solution:**
1. Verify Stable Diffusion WebUI is running: `curl http://localhost:7860`
2. Check if port 7860 is available: `lsof -i :7860`
3. Update `local_url` in config if using different port
4. Ensure `--api` flag is enabled when starting WebUI

### Generation Timeout

**Error:** "API error 504" or "Timeout waiting for response"

**Solution:**
1. Increase `timeout_seconds` in configuration (default: 300)
2. Use lower quality level (less inference steps)
3. Reduce image resolution
4. Ensure GPU/system has adequate resources

### Model Not Found

**Error:** "Model not found" or "Invalid model"

**Solution:**
1. Verify model file exists in WebUI's model directory
2. Check model filename matches configuration
3. Ensure model is downloaded (first launch auto-downloads default)
4. Try downloading SDXL for better quality: [Civitai - SDXL](https://civitai.com/)

### VRAM Issues

**Error:** "CUDA out of memory" or "GPU memory error"

**Solution:**
1. Use smaller resolution (e.g., 512x512 instead of 1024x1024)
2. Reduce inference steps
3. Enable "medvram" or "lowvram" optimizations in WebUI
4. Use CPU mode (slower but uses system RAM instead)

### Cache Issues

**Error:** "Asset cache directory not writable"

**Solution:**
1. Check directory permissions: `ls -la ./generated_assets`
2. Ensure write permissions: `chmod 755 ./generated_assets`
3. Clear cache if corrupted: `rm -rf ./generated_assets/*`

## Performance Tips

1. **Cache Hit**: Enable caching to avoid regenerating identical assets
2. **Batch Generation**: Use faster quality for quick iterations, "best" for final assets
3. **Resolution**: Start with 512x512, scale up only when needed
4. **GPU**: Ensure GPU acceleration is enabled in WebUI settings
5. **Models**: SDXL is slower but produces better quality; use SD 2.1 for faster iterations

## Advanced Configuration

### Custom Model Server

If using a custom Stable Diffusion server:

```toml
[ai_service]
service_type = "local"
local_url = "http://my-custom-server:8080"
local_timeout_seconds = 600
```

### Hugging Face API

For cloud-based generation without local hardware:

```toml
[ai_service]
service_type = "huggingface"
huggingface_api_key = "hf_xxxxxxxxxxxx"
huggingface_model = "stabilityai/stable-diffusion-xl"
```

Or set via environment:
```bash
export HF_API_KEY="hf_xxxxxxxxxxxx"
export HF_MODEL="stabilityai/stable-diffusion-xl"
```

## References

- [AUTOMATIC1111 WebUI](https://github.com/AUTOMATIC1111/stable-diffusion-webui)
- [Stable Diffusion Official](https://huggingface.co/stabilityai)
- [Model Database](https://civitai.com/)
- [API Documentation](https://github.com/AUTOMATIC1111/stable-diffusion-webui/wiki/API)

## Next Steps

1. Set up local Stable Diffusion service using the guide above
2. Update `ai-assets-config.toml` with your setup
3. Test generation using the MCP tools in Claude Code
4. Explore prompt templates and quality levels
5. Integrate generated assets into your game project
