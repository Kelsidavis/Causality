#!/usr/bin/env python3
"""
ComfyUI API Wrapper for Causality Engine
Translates AUTOMATIC1111-style API calls to ComfyUI format
"""

from flask import Flask, request, jsonify, send_file
import requests
import io
import uuid
import os
import json
import base64
import random
from PIL import Image

app = Flask(__name__)

# ComfyUI server URL
COMFYUI_URL = "http://localhost:8188"

# Output directory for generated images
OUTPUT_DIR = "generated_assets/textures"
os.makedirs(OUTPUT_DIR, exist_ok=True)

# Simple workflow template for text-to-image
def create_txt2img_workflow(prompt, width, height, steps=20, cfg=7.5, seed=-1):
    """Create a basic ComfyUI workflow for text-to-image generation"""
    # ComfyUI requires seed >= 0, so generate random seed if -1 is provided
    if seed < 0:
        seed = random.randint(0, 0xFFFFFFFF)

    return {
        "3": {
            "inputs": {"seed": seed, "steps": steps, "cfg": cfg, "sampler_name": "euler", "scheduler": "normal", "denoise": 1, "model": ["4", 0], "positive": ["6", 0], "negative": ["7", 0], "latent_image": ["5", 0]},
            "class_type": "KSampler"
        },
        "4": {
            "inputs": {"ckpt_name": "v1-5-pruned-emaonly.safetensors"},
            "class_type": "CheckpointLoaderSimple"
        },
        "5": {
            "inputs": {"width": width, "height": height, "batch_size": 1},
            "class_type": "EmptyLatentImage"
        },
        "6": {
            "inputs": {"text": prompt, "clip": ["4", 1]},
            "class_type": "CLIPTextEncode"
        },
        "7": {
            "inputs": {"text": "blurry, low quality, distorted, ugly", "clip": ["4", 1]},
            "class_type": "CLIPTextEncode"
        },
        "8": {
            "inputs": {"samples": ["3", 0], "vae": ["4", 2]},
            "class_type": "VAEDecode"
        },
        "9": {
            "inputs": {"filename_prefix": "ComfyUI", "images": ["8", 0]},
            "class_type": "SaveImage"
        }
    }

@app.route('/health', methods=['GET'])
def health():
    """Health check endpoint"""
    try:
        response = requests.get(f"{COMFYUI_URL}/system_stats", timeout=5)
        if response.status_code == 200:
            return jsonify({"status": "ok", "backend": "ComfyUI"}), 200
    except:
        pass
    return jsonify({"status": "error", "message": "ComfyUI not responding"}), 503

@app.route('/api/txt2img', methods=['POST'])
@app.route('/generate-texture', methods=['POST'])
def generate_texture():
    """Generate texture from prompt (AUTOMATIC1111 API compatible)"""
    try:
        data = request.get_json()

        prompt = data.get('prompt', 'texture')
        width = int(data.get('width', 512))
        height = int(data.get('height', 512))
        steps = int(data.get('num_inference_steps', data.get('steps', 20)))
        cfg = float(data.get('guidance_scale', data.get('cfg_scale', 7.5)))
        seed = data.get('seed', -1)

        # Validate and round dimensions to multiples of 8 (ComfyUI requirement)
        width = max(256, min(2048, (width + 7) // 8 * 8))
        height = max(256, min(2048, (height + 7) // 8 * 8))

        print(f"[API Wrapper] Generating: {prompt} ({width}x{height})")

        # Create ComfyUI workflow
        workflow = create_txt2img_workflow(prompt, width, height, steps, cfg, seed)

        # Generate unique client ID and prompt ID
        client_id = str(uuid.uuid4())
        prompt_id = str(uuid.uuid4())

        # Queue the workflow to ComfyUI
        queue_payload = {
            "prompt": workflow,
            "client_id": client_id
        }

        queue_response = requests.post(
            f"{COMFYUI_URL}/prompt",
            json=queue_payload,
            timeout=30
        )

        if queue_response.status_code != 200:
            raise Exception(f"Failed to queue workflow: {queue_response.text}")

        queue_data = queue_response.json()
        actual_prompt_id = queue_data.get("prompt_id")

        print(f"[API Wrapper] Queued prompt {actual_prompt_id}, waiting for completion...")

        # Poll for completion (timeout after 5 minutes)
        import time
        max_wait = 300
        start_time = time.time()

        while time.time() - start_time < max_wait:
            history_response = requests.get(f"{COMFYUI_URL}/history/{actual_prompt_id}")

            if history_response.status_code == 200:
                history = history_response.json()

                if actual_prompt_id in history:
                    outputs = history[actual_prompt_id].get("outputs", {})

                    # Find the SaveImage node output (node 9 in our workflow)
                    if "9" in outputs:
                        images = outputs["9"].get("images", [])
                        if images:
                            # Get the first generated image
                            image_info = images[0]
                            comfy_filename = image_info["filename"]
                            comfy_subfolder = image_info.get("subfolder", "")

                            # Download the image from ComfyUI
                            view_url = f"{COMFYUI_URL}/view"
                            params = {
                                "filename": comfy_filename,
                                "subfolder": comfy_subfolder,
                                "type": "output"
                            }

                            img_response = requests.get(view_url, params=params)

                            if img_response.status_code == 200:
                                # Save to our output directory
                                image_name = f"generated_{uuid.uuid4().hex[:8]}.png"
                                image_path = os.path.join(OUTPUT_DIR, image_name)

                                with open(image_path, 'wb') as f:
                                    f.write(img_response.content)

                                print(f"[API Wrapper] Generated: {image_name}")

                                return jsonify({
                                    "success": True,
                                    "image_name": image_name,
                                    "info": {
                                        "prompt": prompt,
                                        "width": width,
                                        "height": height,
                                        "steps": steps,
                                        "cfg_scale": cfg,
                                        "seed": seed if seed > 0 else 0
                                    }
                                }), 200

            time.sleep(1.0)

        raise Exception("Image generation timed out")

    except Exception as e:
        print(f"[API Wrapper] Error: {e}")
        return jsonify({"error": str(e)}), 500

@app.route('/texture/<image_name>', methods=['GET'])
def get_texture(image_name: str):
    """Retrieve generated texture"""
    try:
        image_path = os.path.join(OUTPUT_DIR, image_name)
        if os.path.exists(image_path):
            return send_file(image_path, mimetype='image/png')
        return jsonify({"error": "Image not found"}), 404
    except Exception as e:
        return jsonify({"error": str(e)}), 500

@app.route('/docs', methods=['GET'])
@app.route('/', methods=['GET'])
def docs():
    """API documentation"""
    return """
    <html>
    <head><title>ComfyUI API Wrapper for Causality Engine</title></head>
    <body>
        <h1>ComfyUI API Wrapper</h1>
        <p>This server wraps ComfyUI to provide AUTOMATIC1111-compatible API endpoints.</p>
        <h2>Endpoints:</h2>
        <ul>
            <li><code>GET /health</code> - Health check</li>
            <li><code>POST /api/txt2img</code> - Generate image from prompt</li>
            <li><code>POST /generate-texture</code> - Generate texture (alias)</li>
            <li><code>GET /texture/&lt;name&gt;</code> - Retrieve generated image</li>
        </ul>
        <h3>Example Request:</h3>
        <pre>
POST /generate-texture
{
    "prompt": "stone brick texture",
    "width": 512,
    "height": 512,
    "steps": 20,
    "guidance_scale": 7.5,
    "seed": 42
}
        </pre>
        <p>ComfyUI Backend: <a href="http://localhost:8188">http://localhost:8188</a></p>
    </body>
    </html>
    """, 200

if __name__ == '__main__':
    print("=" * 80)
    print("ComfyUI API Wrapper for Causality Engine")
    print("=" * 80)
    print()
    print("Starting wrapper on http://0.0.0.0:7860")
    print("ComfyUI backend: http://localhost:8188")
    print()
    print("Endpoints:")
    print("  POST /api/txt2img        - Generate image from prompt")
    print("  POST /generate-texture   - Generate texture (alias)")
    print("  GET  /texture/<name>     - Retrieve generated image")
    print("  GET  /health             - Health check")
    print()

    app.run(host='0.0.0.0', port=7860, debug=False, threaded=True)
