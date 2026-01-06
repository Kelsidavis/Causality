# Texture Prompt Engineering Guide

Guide for creating effective Stable Diffusion prompts for game texture generation.

## Core Principles

### 1. Always Specify "Seamless Tileable"
Game textures must tile seamlessly across surfaces. Always include these keywords:
- `seamless tileable texture pattern`
- `repeating pattern`
- `game texture`

### 2. Avoid Perspective and Structure
Textures should be flat, not 3D objects or scenes:
- ❌ **BAD**: "castle wall", "stone building", "brick wall structure"
- ✅ **GOOD**: "stone brick texture pattern", "weathered bricks surface"

### 3. Use Top-Down or Flat View
Specify the viewing angle to avoid perspective distortion:
- `top-down view`
- `flat view`
- `orthographic view`
- `flat lighting`

### 4. Be Specific About Material
Clearly describe the material properties:
- Material type: stone, wood, metal, fabric, etc.
- Surface details: weathered, smooth, rough, polished
- Color/appearance: gray stone, dark wood, rusty metal

## Prompt Structure Template

```
[MATERIAL TYPE] [SURFACE QUALITY] seamless tileable texture pattern,
[SPECIFIC DETAILS], high detail, game texture,
flat lighting, top-down view, photorealistic
```

### Examples:

**Stone Brick Texture:**
```
seamless tileable stone brick texture pattern, medieval castle bricks,
weathered gray stone blocks, high detail, game texture,
flat lighting, top-down view, photorealistic
```

**Water Texture:**
```
seamless tileable water surface texture, clear turquoise water with ripples,
pool water caustics, high detail, game texture,
flat lighting, top-down view, photorealistic
```

**Grass Texture:**
```
seamless tileable grass texture pattern, lush green grass with soil patches,
medieval terrain, high detail, game texture,
flat lighting, top-down view, photorealistic
```

**Wood Planks:**
```
seamless tileable wood plank texture pattern, weathered oak planks,
wood grain detail, brown wooden boards, high detail, game texture,
flat lighting, top-down view, photorealistic
```

## Negative Prompts

Always include these in negative prompts to avoid common issues:

### Structure & Perspective
- `sky`
- `building`
- `structure`
- `3d render`
- `perspective`
- `architecture`
- `wall` (when generating surface textures)

### Lighting & Shadows
- `shadows`
- `directional lighting`
- `dramatic lighting`

### Environmental Elements
- `grass` (unless it's the texture you want)
- `ground`
- `landscape`
- `horizon`

### Example Negative Prompt:
```
sky, grass, building, structure, 3d render, perspective,
shadows, castle wall, architecture, horizon, dramatic lighting
```

## Common Mistakes

### ❌ Mistake 1: Generating Objects Instead of Textures
**Bad Prompt:** `castle wall with battlements`
- **Result:** Image of actual castle wall structure
- **Fix:** `stone brick texture pattern, medieval castle stones`

### ❌ Mistake 2: Not Specifying Seamless
**Bad Prompt:** `stone bricks`
- **Result:** Non-tileable image with edges that don't match
- **Fix:** `seamless tileable stone brick texture pattern`

### ❌ Mistake 3: Including Environmental Context
**Bad Prompt:** `grass texture on ground with sky`
- **Result:** Landscape photo, not a texture
- **Fix:** `seamless tileable grass texture pattern` + negative prompt `sky, ground, landscape`

### ❌ Mistake 4: Perspective Issues
**Bad Prompt:** `brick wall texture`
- **Result:** Wall photographed at an angle
- **Fix:** `brick texture pattern, flat view, top-down, flat lighting`

## Quality Settings

Recommended settings for texture generation:

- **Resolution:** 512x512 (standard) or 1024x1024 (high detail)
- **Steps:** 20-30 (balance between quality and speed)
- **CFG Scale:** 7.0-8.0 (good prompt adherence without artifacts)
- **Sampler:** Euler or DPM++ 2M Karras

## Testing Textures

After generating a texture:

1. **Visual Inspection:** Check if it looks like a pattern, not a scene
2. **Tile Test:** Imagine the texture repeated - do edges align?
3. **Scale Test:** Does it look good at different scales?
4. **Color Test:** Are colors appropriate for the material?

## Material-Specific Tips

### Stone/Brick
- Specify mortar color: "with white mortar", "with dark mortar"
- Weathering: "weathered", "aged", "clean", "new"
- Stone type: "granite", "limestone", "sandstone"

### Wood
- Grain direction: "horizontal grain", "vertical planks"
- Wood type: "oak", "pine", "mahogany"
- Condition: "polished", "rough-cut", "weathered"

### Metal
- Finish: "brushed", "polished", "rusty", "oxidized"
- Type: "steel", "copper", "bronze", "iron"
- Pattern: "diamond plate", "perforated", "smooth"

### Organic (Grass/Dirt)
- Density: "dense grass", "sparse vegetation"
- Soil visibility: "with soil patches", "pure grass"
- Season/condition: "lush green", "dry brown", "autumn"

### Water
- Clarity: "clear", "murky", "pristine"
- Movement: "still water", "with ripples", "with waves"
- Depth color: "shallow turquoise", "deep blue"

## Troubleshooting

**Problem:** Texture has obvious seams when tiled
- **Solution:** Emphasize "seamless tileable" more, try different seed

**Problem:** Got a 3D object instead of texture
- **Solution:** Add "flat view, top-down" to prompt, add "3d render, structure" to negative prompt

**Problem:** Too much shadow/lighting variation
- **Solution:** Add "flat lighting, even lighting" to prompt, "shadows, dramatic lighting" to negative

**Problem:** Wrong colors
- **Solution:** Be more specific about colors in prompt: "gray stone" not just "stone"

## Version History

- v1.0 (2026-01-05): Initial guide based on castle texture generation experience
  - Learned: Avoid "castle wall" → use "stone brick texture pattern"
  - Learned: Always include "seamless tileable" and "top-down view"
  - Learned: Strong negative prompts prevent structure generation
