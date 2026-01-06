#!/usr/bin/env python3
"""
Mock Stable Diffusion API Server
Emulates AUTOMATIC1111 WebUI API for testing Causality Engine integration
"""

from flask import Flask, request, jsonify, send_file
from PIL import Image
import io
import uuid
import os
from datetime import datetime

app = Flask(__name__)

# Create output directory for generated images
OUTPUT_DIR = "generated_assets/textures"
os.makedirs(OUTPUT_DIR, exist_ok=True)

# Simple in-memory cache for generated images
CACHE = {}

def generate_placeholder_image(width: int, height: int, prompt: str) -> bytes:
    """Generate a placeholder image (solid color based on prompt hash)"""
    # Generate a color based on the prompt
    hash_val = hash(prompt) & 0xFFFFFF
    r = (hash_val >> 16) & 0xFF
    g = (hash_val >> 8) & 0xFF
    b = hash_val & 0xFF

    # Create image
    img = Image.new('RGB', (width, height), (r, g, b))

    # Convert to bytes
    img_bytes = io.BytesIO()
    img.save(img_bytes, format='PNG')
    img_bytes.seek(0)
    return img_bytes.getvalue()

@app.route('/health', methods=['GET'])
def health():
    """Health check endpoint"""
    return jsonify({"status": "ok"}), 200

@app.route('/api/txt2img', methods=['POST'])
def txt2img():
    """Generate image from text prompt"""
    try:
        data = request.get_json()

        prompt = data.get('prompt', 'texture')
        width = int(data.get('width', 512))
        height = int(data.get('height', 512))
        steps = int(data.get('steps', 50))
        seed = data.get('seed')

        # Validate dimensions
        if width < 256 or width > 2048:
            width = max(256, min(2048, width))
        if height < 256 or height > 2048:
            height = max(256, min(2048, height))

        # Round to nearest multiple of 64
        width = ((width + 31) // 64) * 64
        height = ((height + 31) // 64) * 64

        print(f"[Mock API] Generating image: {prompt} ({width}x{height})")

        # Generate placeholder image
        image_data = generate_placeholder_image(width, height, prompt)

        # Save image
        image_name = f"generated_{uuid.uuid4().hex[:8]}.png"
        image_path = os.path.join(OUTPUT_DIR, image_name)

        with open(image_path, 'wb') as f:
            f.write(image_data)

        print(f"[Mock API] Saved image to {image_path}")

        # Cache it
        CACHE[image_name] = {
            'data': image_data,
            'prompt': prompt,
            'width': width,
            'height': height,
            'steps': steps,
            'seed': seed,
            'timestamp': datetime.now().isoformat()
        }

        return jsonify({
            "info": {
                "prompt": prompt,
                "all_prompts": [prompt],
                "negative_prompts": [""],
                "seed": seed or 0,
                "all_seeds": [seed or 0],
                "subseed": -1,
                "subseed_strength": 0,
                "width": width,
                "height": height,
                "cfg_scale": 7.5,
                "steps": steps,
                "sampler_name": "DPM++ 2M Karras",
                "scheduler": "karras",
                "seed_resize_from_h": -1,
                "seed_resize_from_w": -1,
                "denoising_strength": 1.0,
                "extra_generation_params": {},
                "index_of_first_image": 0,
                "infotexts": [f"{prompt}"],
                "styles": [],
                "job_timestamp": datetime.now().isoformat(),
                "clip_skip": -1,
                "is_using_inpaint_model": False,
                "version": "v1.10.1 (mock)"
            },
            "images": ["data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg=="],
            "image_name": image_name
        }), 200

    except Exception as e:
        print(f"[Mock API] Error: {e}")
        return jsonify({"error": str(e)}), 500

@app.route('/texture/<image_name>', methods=['GET'])
def get_texture(image_name: str):
    """Retrieve generated texture"""
    try:
        # Check cache first
        if image_name in CACHE:
            image_data = CACHE[image_name]['data']
            return send_file(
                io.BytesIO(image_data),
                mimetype='image/png',
                as_attachment=False
            )

        # Check file system
        image_path = os.path.join(OUTPUT_DIR, image_name)
        if os.path.exists(image_path):
            return send_file(image_path, mimetype='image/png')

        return jsonify({"error": "Image not found"}), 404

    except Exception as e:
        print(f"[Mock API] Error retrieving texture: {e}")
        return jsonify({"error": str(e)}), 500

@app.route('/sdapi/v1/txt2img', methods=['POST'])
def sdapi_txt2img():
    """Alternative API endpoint (v1)"""
    return txt2img()

@app.route('/info', methods=['GET'])
def info():
    """Server info endpoint"""
    return jsonify({
        "version": "mock-1.0",
        "app_id": "stable-diffusion-webui",
        "config": {},
        "models": [
            {
                "title": "Stable Diffusion 2.1 (Mock)",
                "model_name": "sd-v2-1",
                "hash": "mock",
                "sha256": "mock",
                "config": None,
                "config_name": "v2-1_768-ema-pruned.safetensors",
                "epoch": None,
                "step": None
            }
        ]
    }), 200

if __name__ == '__main__':
    print("=" * 80)
    print("Mock Stable Diffusion API Server")
    print("=" * 80)
    print()
    print("Starting server on http://0.0.0.0:7860")
    print("Endpoints:")
    print("  POST /api/txt2img        - Generate image from prompt")
    print("  GET  /texture/<name>     - Retrieve generated image")
    print("  GET  /health             - Health check")
    print("  GET  /info               - Server info")
    print()
    print("This mock server generates placeholder images for testing.")
    print("Colors are deterministically generated from prompt hashes.")
    print()

    app.run(host='0.0.0.0', port=7860, debug=False, threaded=True)
