# AI Music Generation Demo

This demonstrates the MCP-based music generation workflow.

## Request Example

When the game engine (or user) requests music generation, here's what happens:

### 1. Game Engine Sends MCP Request

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "generate_music",
    "arguments": {
      "prompt": "Uplifting instrumental music for exploring a sunny world, light acoustic guitar, soft piano, cheerful melody",
      "output_path": "assets/music/sunny_exploration.wav",
      "duration": "long",
      "style": "acoustic",
      "tempo": 110,
      "instrumental": true
    }
  }
}
```

### 2. MCP Server Responds to Claude

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "content": [{
      "type": "text",
      "text": "Music generation request received:
Prompt: Uplifting instrumental music for exploring a sunny world, light acoustic guitar, soft piano, cheerful melody
Duration: long
Style: acoustic
Tempo: 110
Instrumental: true
Output: assets/music/sunny_exploration.wav

Please use the ACE-Step API at http://localhost:7865 to generate this music.
Then save the generated audio to 'assets/music/sunny_exploration.wav' and confirm completion."
    }]
  }
}
```

### 3. Claude (Me) Would Call ACE-Step

```bash
# HTTP POST to ACE-Step API
curl -X POST http://localhost:7865/generate \
  -H "Content-Type: application/json" \
  -d '{
    "prompt": "Uplifting instrumental music for exploring a sunny world, light acoustic guitar, soft piano, cheerful melody",
    "duration": 60,
    "style": "acoustic",
    "tempo": 110,
    "instrumental": true,
    "inference_steps": 27
  }' \
  --output sunny_exploration.wav
```

### 4. ACE-Step Generates Music

- Loads the AI model (~2GB)
- Runs inference (20-60 seconds depending on hardware)
- Returns WAV audio file

### 5. Claude Saves and Confirms

```
I've generated a beautiful sunny exploration track!

File: assets/music/sunny_exploration.wav
Duration: 60 seconds
Style: Acoustic instrumental
Features: Light acoustic guitar, soft piano, cheerful uplifting melody
Perfect for: Exploring sunny open-world environments

The music is ready to play in the game!
```

### 6. Game Engine Plays Music (Optional)

```json
{
  "method": "tools/call",
  "params": {
    "name": "play_music",
    "arguments": {
      "file_path": "assets/music/sunny_exploration.wav",
      "loop": true,
      "volume": 0.7,
      "fade_in": 2.0
    }
  }
}
```

## Current Status

**MCP Integration**: ✅ Complete and working
**ACE-Step Setup**: ⚠️ Requires full installation

To fully test this:

1. **Install ACE-Step dependencies** (one-time setup):
   ```bash
   cd ACE-Step
   source venv/bin/activate
   pip install torch torchvision torchaudio --index-url https://download.pytorch.org/whl/cpu
   pip install -r requirements.txt
   ```

2. **Start ACE-Step server**:
   ```bash
   acestep --port 7865
   ```

3. **Make request through MCP** (or ask Claude directly):
   "Generate some cheerful exploration music"

## Benefits of MCP Approach

1. **Claude handles complexity**: I manage ACE-Step API calls, error handling, retries
2. **Intelligent prompts**: I can enhance prompts based on context
3. **Flexible**: Can use ACE-Step, MusicGen, or other services
4. **No coupling**: Game doesn't depend on specific music API
5. **Centralized AI**: All AI operations go through one protocol

## Alternative: Direct API

For immediate testing without MCP, you can use the `engine-ai-music` crate:

```rust
use engine_ai_music::prelude::*;

let client = AceStepClient::new();
let music = client.generate_music(
    MusicGenerationRequest::new("Sunny exploration music")
        .with_duration(MusicDuration::Long)
        .with_style(MusicStyle::Acoustic)
).await?;
```

But MCP is recommended for production as it's more flexible and intelligent.
