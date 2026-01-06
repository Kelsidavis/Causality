#!/bin/bash
# Integration test for Stable Diffusion setup with Causality Engine

echo "=========================================="
echo "Stable Diffusion Integration Test"
echo "=========================================="
echo

# Check ComfyUI is running
echo "1. Checking ComfyUI backend..."
if curl -s http://localhost:8188/system_stats > /dev/null 2>&1; then
    echo "   ✓ ComfyUI is running on port 8188"
else
    echo "   ✗ ComfyUI is not responding"
    exit 1
fi
echo

# Check API Wrapper is running
echo "2. Checking API Wrapper..."
HEALTH=$(curl -s http://localhost:7860/health)
if echo "$HEALTH" | grep -q "ok"; then
    echo "   ✓ API Wrapper is running on port 7860"
    echo "   Response: $HEALTH"
else
    echo "   ✗ API Wrapper is not responding"
    exit 1
fi
echo

# Generate a test texture
echo "3. Generating test texture..."
RESULT=$(curl -s -X POST http://localhost:7860/generate-texture \
  -H "Content-Type: application/json" \
  -d '{
    "prompt": "wooden planks texture, seamless tileable",
    "width": 512,
    "height": 512,
    "steps": 15,
    "guidance_scale": 7.0
  }')

if echo "$RESULT" | grep -q "success.*true"; then
    IMAGE_NAME=$(echo "$RESULT" | grep -o '"image_name":"[^"]*"' | cut -d'"' -f4)
    echo "   ✓ Texture generated successfully"
    echo "   Image: $IMAGE_NAME"

    # Verify image file exists
    IMAGE_PATH="/home/k/game-engine/generated_assets/textures/$IMAGE_NAME"
    if [ -f "$IMAGE_PATH" ]; then
        SIZE=$(du -h "$IMAGE_PATH" | cut -f1)
        DIMS=$(file "$IMAGE_PATH" | grep -o '[0-9]* x [0-9]*')
        echo "   ✓ Image file verified: $SIZE, $DIMS"
    else
        echo "   ✗ Image file not found: $IMAGE_PATH"
        exit 1
    fi
else
    echo "   ✗ Texture generation failed"
    echo "   Response: $RESULT"
    exit 1
fi
echo

echo "=========================================="
echo "All tests passed! ✓"
echo "=========================================="
echo
echo "Stable Diffusion is fully integrated with Causality Engine."
echo
echo "Usage from Rust code:"
echo "  use engine_ai_assets::{LocalClient, AssetGenerator, AssetCache};"
echo "  let client = LocalClient::localhost(7860, 300);"
echo "  let cache = AssetCache::new(\"./generated_assets\")?;"
echo "  let generator = AssetGenerator::new(Box::new(client), cache)?;"
echo
