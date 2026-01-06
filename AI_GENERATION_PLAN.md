# Stable Diffusion Integration Plan

## Overview

Add AI-powered texture and asset generation to Causality Engine using Stable Diffusion, enabling Claude to create custom textures, skyboxes, and concept art through natural language descriptions.

## Architecture

### Three Implementation Approaches

#### 1. **API-Based (Recommended - Phase 1)**
- Use Hugging Face Inference API or Replicate
- No local GPU required
- Users provide API key
- Instant deployment
- Limitation: API costs

#### 2. **Local Stable Diffusion (Phase 2)**
- Local installation (ComfyUI or PyTorch)
- Zero API costs
- Requires CUDA GPU
- Better for power users
- More setup required

#### 3. **Hybrid (Phase 3)**
- Try local first, fallback to API
- Best of both worlds
- Flexible for different user setups

## Phase 1: API-Based Integration

### MCP Tools to Implement

1. **generate_texture**
   - Input: Prompt, width, height, seed (optional)
   - Output: Generated texture saved and returned
   - Example: "A rough stone wall texture with moss"

2. **generate_skybox**
   - Input: Prompt (scene description)
   - Output: 6 cubemap faces (or cross texture)
   - Example: "A serene sunset over mountains"

3. **refine_texture**
   - Input: Existing texture + modification prompt
   - Output: Modified texture via inpainting
   - Example: "Add more rust to the metal texture"

4. **generate_material_set**
   - Input: Material description
   - Output: Albedo, normal, roughness, metallic textures
   - Example: "Weathered wooden planks"

### Component Architecture

```
┌─────────────────────────────────────────────┐
│         Causality Engine Texture Gen         │
├─────────────────────────────────────────────┤
│                                              │
│  ┌──────────────────────────────────────┐   │
│  │  MCP Tools (engine-mcp-server)       │   │
│  │  - generate_texture                  │   │
│  │  - generate_skybox                   │   │
│  │  - refine_texture                    │   │
│  └──────────────────────────────────────┘   │
│            ↓                                  │
│  ┌──────────────────────────────────────┐   │
│  │  AIAssetGenerator (new crate)        │   │
│  │  - API client (Hugging Face/Replicate)  │
│  │  - Prompt processing                 │   │
│  │  - Response handling                 │   │
│  └──────────────────────────────────────┘   │
│            ↓                                  │
│  ┌──────────────────────────────────────┐   │
│  │  Asset Cache System                  │   │
│  │  - Disk cache (generated_assets/)    │   │
│  │  - Hot reload integration            │   │
│  │  - Metadata tracking                 │   │
│  └──────────────────────────────────────┘   │
│            ↓                                  │
│  ┌──────────────────────────────────────┐   │
│  │  Engine Assets                       │   │
│  │  - Texture loading                   │   │
│  │  - Asset management                  │   │
│  └──────────────────────────────────────┘   │
└─────────────────────────────────────────────┘
```

## Implementation Steps

### Step 1: Create AI Asset Generator Crate

**Crate Structure** (`engine-ai-assets/`):
```
src/
├── lib.rs                 # Public API
├── generator.rs           # Main texture generator
├── api/
│   ├── mod.rs
│   ├── huggingface.rs    # Hugging Face API client
│   └── replicate.rs      # Replicate API client (future)
├── cache.rs              # Asset caching system
├── prompt.rs             # Prompt engineering & optimization
└── models.rs             # Generation models registry
```

### Step 2: Implement API Clients

**Hugging Face Inference API**:
- Endpoint: `https://api-inference.huggingface.co/models/...`
- Models: Stable Diffusion 1.5, 2.1, XL, SDXL
- Features: Text-to-image, inpainting, upscaling
- Auth: Bearer token via environment variable

**Response Flow**:
```
Prompt → Validation → API Call → Image Bytes → Save to Cache → Asset Manager
```

### Step 3: Texture Generation Features

**Texture Variants**:
- **Color/Albedo**: Base color texture (2K-4K)
- **Normal Map**: Surface normal details
- **Roughness**: PBR roughness map
- **Metallic**: PBR metallic mask
- **Ambient Occlusion**: AO baking
- **Skyboxes**: 6-face cubemaps

**Prompt Enhancement**:
- Auto-format prompts for better results
- Add quality keywords: "high quality", "detailed", "4k"
- Add style hints: "photorealistic", "game-ready"
- Negative prompts: "blurry", "low quality"

### Step 4: Cache System

**Cache Structure**:
```
generated_assets/
├── metadata.json              # All generations metadata
├── textures/
│   ├── stone_wall_moss_01.png
│   ├── stone_wall_moss_02.png
│   └── ...
├── skyboxes/
│   ├── sunset_mountains_01/
│   │   ├── px.png
│   │   ├── nx.png
│   │   ├── py.png
│   │   ├── ny.png
│   │   ├── pz.png
│   │   └── nz.png
│   └── ...
└── generation_log.jsonl       # Audit trail
```

**Metadata Example**:
```json
{
  "id": "uuid-here",
  "prompt": "A rough stone wall texture with moss",
  "model": "stabilityai/stable-diffusion-2-1",
  "timestamp": "2024-01-15T10:30:00Z",
  "width": 2048,
  "height": 2048,
  "seed": 12345,
  "api_cost": 0.002,
  "file_path": "generated_assets/textures/stone_wall_moss_01.png"
}
```

### Step 5: MCP Tool Implementation

**Tool Signatures**:

```rust
async fn generate_texture(
  prompt: String,
  width: u32,
  height: u32,
  model: Option<String>,
  seed: Option<u64>,
) -> Result<GeneratedAsset>

async fn generate_skybox(
  prompt: String,
  size: u32,
) -> Result<SkyboxAsset>

async fn refine_texture(
  texture_path: String,
  modification_prompt: String,
  inpaint_mask: Option<String>,
) -> Result<GeneratedAsset>

async fn list_generated_assets(
  asset_type: Option<String>,
) -> Result<Vec<AssetMetadata>>
```

## Configuration

### Environment Variables

```bash
# Required
HF_API_KEY=hf_xxxx...                    # Hugging Face API token
HF_MODEL=stabilityai/stable-diffusion-2-1  # Model selection

# Optional
HF_TIMEOUT=300                           # API timeout in seconds
TEXTURE_CACHE_DIR=generated_assets/      # Cache location
TEXTURE_QUALITY=best                     # best|high|standard|fast
UPSCALER_ENABLED=true                    # Enable 4x upscaling
```

### MCP Server Config

```json
{
  "claude-code": {
    "mcp_config": {
      "ai-asset-generation": {
        "enabled": true,
        "api_key_env": "HF_API_KEY",
        "cache_dir": "./generated_assets"
      }
    }
  }
}
```

## Workflow Examples

### Example 1: Generate Stone Texture

```python
# Claude generates a texture
generate_texture(
  prompt="Rough stone wall texture with moss and lichen",
  width=2048,
  height=2048
)

# Returns:
# - Texture saved to generated_assets/textures/
# - Asset metadata updated
# - Ready to use in editor
```

### Example 2: Create Skybox

```python
# Generate a beautiful skybox
generate_skybox(
  prompt="Serene sunset over calm ocean, golden hour, soft clouds"
)

# Returns:
# - 6 cubemap faces or cross texture
# - Saved to generated_assets/skyboxes/
# - Can be immediately applied to scene
```

### Example 3: Interactive Texture Refinement

```python
# Generate initial texture
texture = generate_texture("Metal surface")

# User asks for modifications
refined = refine_texture(
  texture_path=texture.path,
  modification_prompt="Add more rust and oxidation"
)

# Result: Iterative texture improvement
```

## Dependencies

Add to `Cargo.toml`:

```toml
[dependencies]
reqwest = { version = "0.11", features = ["json", "multipart"] }
tokio = { version = "1", features = ["full"] }
image = "0.25"
serde_json = "1.0"
uuid = { version = "1.0", features = ["v4", "serde"] }
anyhow = "1.0"
log = "0.4"
sha2 = "0.10"  # For caching
```

## Cost Estimation

**Hugging Face Inference API** (Recommended):
- Stable Diffusion: $0.001 - $0.005 per image
- SDXL: $0.004 - $0.012 per image
- Upscaling: $0.001 - $0.002 per image
- **100 textures: ~$0.50**

**Replicate** (Alternative):
- Similar pricing, slightly higher
- Better uptime reliability

**Local Stable Diffusion** (Phase 2):
- One-time GPU setup
- Zero per-image costs
- ~15-30 seconds per generation

## Security Considerations

1. **API Key Management**:
   - Store in environment variables only
   - Never log or expose keys
   - Support key rotation

2. **Prompt Injection**:
   - Validate and sanitize user prompts
   - Limit prompt length (<500 chars)
   - Filter malicious keywords

3. **Rate Limiting**:
   - Implement per-user rate limits
   - Cache identical prompts
   - Throttle API calls

4. **Generated Content**:
   - Respect model licenses (Stability AI, SDXL)
   - Attribute generated assets appropriately
   - Track generation provenance

## Testing Strategy

1. **Unit Tests**:
   - Prompt formatting
   - Cache validation
   - API mock testing

2. **Integration Tests**:
   - Real Hugging Face API calls (with test account)
   - Full texture generation pipeline
   - Caching behavior

3. **Manual Testing**:
   - Generate various textures
   - Test cache hit/miss
   - Verify hot-reload integration

## Future Enhancements

- **Local GPU Support**: ComfyUI integration for power users
- **Advanced Models**: SDXL, custom LoRA fine-tunes
- **Batch Generation**: Generate texture sets in parallel
- **Style Transfer**: Apply styles to existing textures
- **3D Generation**: Integrate with text-to-3D models
- **Real-time Preview**: Live texture generation in editor
- **Upscaling**: 2x/4x super-resolution
- **Model Fine-tuning**: Train custom models on user data

## Timeline

- **Week 1**: Hugging Face API client + basic texture generation
- **Week 2**: Caching system + metadata tracking
- **Week 3**: MCP tool implementation + testing
- **Week 4**: Documentation + local SD support
- **Ongoing**: Advanced features + optimizations

## Success Criteria

✅ Generate quality textures from text descriptions
✅ All generations cached and indexed
✅ Full MCP tool integration
✅ Sub-5 second API responses
✅ <2 second cache hits
✅ Zero API key leaks in logs
✅ Comprehensive documentation
✅ Working examples for users

---

## Implementation Ready

This plan provides a clear path to add powerful AI-powered asset generation to Causality Engine. Users will be able to ask Claude to "create a stone texture" and have it appear instantly in the editor!
