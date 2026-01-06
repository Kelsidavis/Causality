# AI Asset Generation Examples for Causality Engine

This document provides practical examples for generating game assets using Stable Diffusion with the Causality Engine.

## Table of Contents

1. [Basic Texture Generation](#basic-texture-generation)
2. [Skybox Generation](#skybox-generation)
3. [Material Workflows](#material-workflows)
4. [Using with Claude Code](#using-with-claude-code)
5. [Programmatic Examples](#programmatic-examples)

## Basic Texture Generation

### Example 1: Stone Wall Texture

Using Claude Code MCP tools:

```
generate_texture(
  prompt: "seamless stone wall texture, rough surface, aged appearance, high quality, 4k"
  width: 512
  height: 512
  quality: "high"
  seed: 12345
)
```

Result:
- Asset ID: `abc123def456...`
- File path: `textures/stone_wall_<uuid>.png`
- Cached for future use

### Example 2: Wooden Floor

```
generate_texture(
  prompt: "wooden plank floor texture, hardwood, aged wood grain, seamless, pbr, 4k"
  width: 1024
  height: 1024
  quality: "best"
)
```

Benefits of higher resolution:
- Better detail for close-up viewing
- More suitable for large surfaces
- Higher quality generation (75 steps instead of 50)

### Example 3: Quick Preview (Fast Quality)

```
generate_texture(
  prompt: "brick wall texture"
  width: 512
  height: 512
  quality: "fast"
)
```

Fast generation (20 steps) is useful for:
- Previewing prompts before high-quality generation
- Rapid iteration during development
- Testing different materials

## Skybox Generation

### Example 1: Sunset Sky

```
generate_skybox(
  prompt: "beautiful golden sunset, dramatic orange and pink clouds, clear sky, peaceful"
  quality: "high"
  seed: 54321
)
```

Result:
- Resolution: 2048x1024 (360° panorama)
- File: `skyboxes/sunset_<uuid>.png`
- Covers entire scene environment

### Example 2: Forest Environment

```
generate_skybox(
  prompt: "lush forest clearing, tall trees, dappled sunlight, blue sky, nature scene"
  quality: "best"
)
```

### Example 3: Cyberpunk City

```
generate_skybox(
  prompt: "futuristic cyberpunk cityscape at night, neon lights, flying vehicles, skyscrapers, dystopian atmosphere"
  quality: "high"
)
```

## Material Workflows

### PBR Texture Set Workflow

Generate a complete PBR material set using multiple texture generation calls:

```
// Base Color
generate_texture(
  prompt: "rusty metal surface, realistic rust texture, orangish-brown color, weathered steel, 4k, pbr"
  width: 1024
  height: 1024
  quality: "best"
)

// Roughness Map (grayscale)
generate_texture(
  prompt: "grayscale roughness map, rusty metal surface, varied surface detail, pbr"
  width: 1024
  height: 1024
  quality: "high"
)

// Normal Map (blue-ish)
generate_texture(
  prompt: "blue normal map, rusty metal surface relief, detailed bumps, tangent space normals"
  width: 1024
  height: 1024
  quality: "high"
)
```

### Tileable Texture Pattern

For repeating game surfaces:

```
generate_texture(
  prompt: "seamless tileable pattern, geometric, clean edges, modern design, no seams visible"
  width: 512
  height: 512
  quality: "high"
)
```

Tips for tileable textures:
- Use "seamless" and "no seams" in prompt
- Avoid unique features that would be obvious when tiled
- Test by creating multiple copies side-by-side

### Character Skin Texture

```
generate_texture(
  prompt: "realistic human skin texture, detailed pores, subtle color variation, clean, face texture"
  width: 2048
  height: 2048
  quality: "best"
)
```

## Using with Claude Code

### Scenario 1: Generate Texture for Imported Model

```
# Using Claude Code with engine-mcp-server

generate_texture(
  prompt: "weathered wooden barrel texture, aged oak wood, realistic wood grain"
  width: 512
  height: 512
  quality: "high"
)

# Then load the model and apply the texture
load_model(
  path: "assets/models/barrel.gltf"
  scale: [1.0, 1.0, 1.0]
  position: [0, 0, -5]
)

# Apply the generated texture to material
# (texture applied to the newly created entity)
```

### Scenario 2: Quick Scene Setup with Generated Assets

```
# Generate environment
generate_skybox(
  prompt: "peaceful mountain valley, snow-capped peaks, clear blue sky"
  quality: "high"
)

# Generate ground texture
generate_texture(
  prompt: "grass field texture, green meadow, seamless, high quality"
  width: 1024
  height: 1024
  quality: "high"
)

# Create entities
create_entity(
  name: "Ground"
  position: [0, 0, 0]
)

# Add physics
add_rigidbody(
  entity_name: "Ground"
  body_type: "static"
)
```

## Programmatic Examples

### Rust Example: Basic Generation

```rust
use engine_ai_assets::{AssetGenerator, AssetCache, TextureGenerationRequest, LocalClient};

#[tokio::main]
async fn generate_texture_example() -> anyhow::Result<()> {
    // Create client for local Stable Diffusion service
    let client = LocalClient::localhost(7860, 300);

    // Create cache system
    let cache = AssetCache::new("./generated_assets")?;

    // Create generator
    let generator = AssetGenerator::new(Box::new(client), cache)?;

    // Create request
    let request = TextureGenerationRequest {
        prompt: "stone brick wall texture, rough surface, weathered".to_string(),
        negative_prompt: Some("smooth, shiny, clean".to_string()),
        width: 512,
        height: 512,
        steps: 50,
        guidance_scale: 7.5,
        seed: Some(12345),
        use_cache: true,
    };

    // Generate
    let asset = generator.generate_texture(&request).await?;

    println!("Generated texture:");
    println!("  ID: {}", asset.metadata.id);
    println!("  File: {}", asset.metadata.file_path);
    println!("  From cache: {}", asset.from_cache);
    println!("  Size: {} bytes", asset.metadata.file_size);

    Ok(())
}
```

### Rust Example: Batch Generation

```rust
use engine_ai_assets::{AssetGenerator, AssetCache, TextureGenerationRequest, LocalClient};

#[tokio::main]
async fn batch_generate_textures() -> anyhow::Result<()> {
    let client = LocalClient::localhost(7860, 300);
    let cache = AssetCache::new("./generated_assets")?;
    let generator = AssetGenerator::new(Box::new(client), cache)?;

    let material_prompts = vec![
        ("Stone", "stone brick texture, rough, weathered"),
        ("Wood", "wooden plank texture, hardwood, grain patterns"),
        ("Metal", "metal surface texture, brushed steel, realistic"),
        ("Grass", "grass field texture, green meadow, seamless"),
    ];

    for (name, prompt) in material_prompts {
        let request = TextureGenerationRequest {
            prompt: prompt.to_string(),
            negative_prompt: Some("blurry, low quality".to_string()),
            width: 512,
            height: 512,
            steps: 50,
            guidance_scale: 7.5,
            seed: None,
            use_cache: true,
        };

        match generator.generate_texture(&request).await {
            Ok(asset) => {
                println!("✓ Generated {}: {}", name, asset.metadata.file_path);
            }
            Err(e) => {
                eprintln!("✗ Failed to generate {}: {}", name, e);
            }
        }
    }

    Ok(())
}
```

### Rust Example: Skybox Generation

```rust
use engine_ai_assets::{AssetGenerator, AssetCache, LocalClient};

#[tokio::main]
async fn generate_skybox_example() -> anyhow::Result<()> {
    let client = LocalClient::localhost(7860, 300);
    let cache = AssetCache::new("./generated_assets")?;
    let generator = AssetGenerator::new(Box::new(client), cache)?;

    let environments = vec![
        "beautiful sunset, orange and pink sky",
        "sunny day, clear blue sky, white clouds",
        "stormy weather, dark clouds, dramatic lighting",
    ];

    for env in environments {
        let asset = generator.generate_skybox(env, Some(42)).await?;
        println!("Generated skybox: {}", asset.metadata.file_path);
    }

    Ok(())
}
```

### Rust Example: Prompt Optimization

```rust
use engine_ai_assets::prompt::{PromptOptimizer, QualityLevel, templates, styles};

fn prompt_optimization_example() {
    // Using quality levels
    let optimizer = PromptOptimizer::new(QualityLevel::High);
    let optimized = optimizer.optimize("stone wall");

    println!("Original: stone wall");
    println!("Optimized: {}", optimized.prompt);
    println!("Negative: {}", optimized.negative_prompt);
    println!("Steps: {}", optimized.steps);
    println!("Guidance: {}", optimized.guidance_scale);

    // Using templates
    let pbr_prompt = templates::pbr_texture("wood", "aged");
    println!("\nPBR Prompt: {}", pbr_prompt);

    let game_prompt = templates::game_texture("stone", "clean");
    println!("Game Prompt: {}", game_prompt);

    // Using style modifiers
    let styled = format!(
        "{}, {}",
        "stone brick texture",
        styles::pbr_material()
    );
    println!("Styled: {}", styled);
}
```

## Quality Comparison

### "fast" Quality - Quick Iteration
```
Inference Steps: 20
Guidance Scale: 5.0
Time: ~10-15 seconds
Use Case: Preview and testing
```

### "standard" Quality - Balanced
```
Inference Steps: 35
Guidance Scale: 7.0
Time: ~20-30 seconds
Use Case: Development assets
```

### "high" Quality - Recommended
```
Inference Steps: 50
Guidance Scale: 7.5
Time: ~30-45 seconds
Use Case: Production assets (default)
```

### "best" Quality - Maximum
```
Inference Steps: 75
Guidance Scale: 8.5
Time: ~60-90 seconds
Use Case: Final/showcase assets
```

## Cache Management

### Check Cache Status

```rust
let stats = generator.cache_stats()?;
println!("Total assets: {}", stats.total_assets);
println!("Total size: {} bytes", stats.total_size);
println!("Textures: {}", stats.textures);
println!("Skyboxes: {}", stats.skyboxes);
```

### Clear Cache

```rust
generator.clear_cache()?;
println!("Cache cleared");
```

### Disable Cache for Generation

```rust
let request = TextureGenerationRequest {
    prompt: "test texture".to_string(),
    // ... other fields
    use_cache: false,  // Always generate fresh
};
```

## Tips and Best Practices

### Prompt Engineering

1. **Be Specific**: "weathered oak plank texture with visible grain" > "wood"
2. **Include Quality Keywords**: "high quality", "detailed", "4k", "professional"
3. **Use Negative Prompts**: Specify what you DON'T want
4. **Add Style Keywords**: "pbr", "seamless", "tileable" for game assets
5. **Reference Quality**: "masterpiece", "best quality" for highest tier

### Iteration Strategy

1. Start with "fast" quality to test prompt effectiveness
2. Refine prompt based on results
3. Use "high" quality for final version
4. Cache successful results for reuse

### Performance Optimization

1. **Resolution**: 512x512 is usually sufficient for game textures
2. **Batch Generation**: Generate multiple assets in sequence
3. **Seed Control**: Use seeds to reproduce exact results
4. **Quality Levels**: Adjust based on deadline vs. quality needs

## Troubleshooting

### Prompt Not Working Well

**Solution:** Try the built-in templates:
```rust
let prompt = templates::pbr_texture("stone", "weathered");
generator.generate_texture(&request).await?;
```

### Generation Too Slow

**Solutions:**
1. Reduce resolution (512x512 vs 1024x1024)
2. Use "fast" or "standard" quality
3. Reduce inference steps in config
4. Ensure GPU acceleration is enabled

### Cache Not Working

**Solution:** Verify cache directory exists and is writable:
```bash
mkdir -p ./generated_assets/{textures,skyboxes,metadata}
chmod 755 ./generated_assets
```

## Advanced Topics

### Custom Models

To use different Stable Diffusion models, configure in `ai-assets-config.toml`:

```toml
[ai_service]
huggingface_model = "stabilityai/stable-diffusion-xl"
```

Then regenerate with the new model - the cache system will track model-specific results.

### Upscaling

Enable upscaling for 4K textures (requires server-side support):

```toml
[generation]
upscaler_enabled = true
```

### External API Integration

For distributed generation across machines, modify the local URL to point to a remote server:

```toml
[ai_service]
local_url = "http://192.168.1.100:7860"
```
