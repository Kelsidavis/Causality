# AI Music Generation Through MCP

The game engine uses the **Model Context Protocol (MCP)** for AI-powered music generation. This allows Claude or other AI assistants to handle music generation requests and use external services like ACE-Step.

## Architecture

```
Game Engine         MCP Server          Claude/AI           ACE-Step API
    │                   │                   │                     │
    │ generate_music    │                   │                     │
    ├──────────────────►│                   │                     │
    │   (MCP request)   │                   │                     │
    │                   │  Call tool        │                     │
    │                   ├──────────────────►│                     │
    │                   │                   │  WebFetch/HTTP      │
    │                   │                   ├────────────────────►│
    │                   │                   │  POST /generate     │
    │                   │                   │                     │
    │                   │                   │◄────────────────────┤
    │                   │                   │  Audio data         │
    │                   │  Tool response    │                     │
    │                   │◄──────────────────┤                     │
    │◄──────────────────┤                   │                     │
    │  Success/file path│                   │                     │
    │                   │                   │                     │
```

**Benefits of MCP Approach:**
- Claude handles the complexity of calling ACE-Step
- No direct API dependencies in game code
- Claude can use context and intelligence to improve prompts
- Easy to swap or add alternative music generation services
- Centralized AI operations through single protocol

## MCP Tools

The MCP server provides three music-related tools:

### 1. generate_music

Generate music from a text description.

**Parameters:**
- `prompt` (required): Text description of music (e.g., "Epic battle music with drums")
- `output_path` (required): Where to save the file (e.g., "assets/music/battle.wav")
- `duration`: "short" (~15s), "medium" (~30s), "long" (~60s), or "extended" (~2min)
- `style`: Genre like "rock", "pop", "electronic", "cinematic", etc.
- `tempo`: BPM (e.g., 120)
- `instrumental`: true for no vocals (default: true)

**Example MCP Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "generate_music",
    "arguments": {
      "prompt": "Epic orchestral battle music with drums and brass",
      "output_path": "assets/music/battle.wav",
      "duration": "medium",
      "style": "cinematic",
      "instrumental": true
    }
  }
}
```

**What Claude Does:**
1. Receives the request via MCP
2. Uses WebFetch or HTTP tools to call ACE-Step API at `http://localhost:7865`
3. Sends appropriate parameters to ACE-Step
4. Downloads the generated audio
5. Saves it to the specified output path
6. Returns success confirmation to game engine

### 2. play_music

Play a music file in the game.

**Parameters:**
- `file_path` (required): Path to music file
- `loop`: Whether to loop (default: true)
- `volume`: 0.0 to 1.0 (default: 0.8)
- `fade_in`: Fade in duration in seconds (default: 0.0)

**Example:**
```json
{
  "method": "tools/call",
  "params": {
    "name": "play_music",
    "arguments": {
      "file_path": "assets/music/battle.wav",
      "loop": true,
      "volume": 0.9,
      "fade_in": 2.0
    }
  }
}
```

### 3. stop_music

Stop currently playing music.

**Parameters:**
- `fade_out`: Fade out duration in seconds (default: 0.0 for immediate stop)

**Example:**
```json
{
  "method": "tools/call",
  "params": {
    "name": "stop_music",
    "arguments": {
      "fade_out": 3.0
    }
  }
}
```

## Setup

### 1. Start ACE-Step Service

In a separate terminal:

```bash
cd ACE-Step
source venv/bin/activate
acestep --port 7865
```

### 2. Start MCP Server

The MCP server should be running and connected to Claude Code:

```bash
cargo run --bin engine-mcp-server
```

### 3. Start Game Engine/Editor

```bash
cargo run --bin engine-editor
```

### 4. Use Through Claude

When Claude Code is connected to the MCP server, you can simply ask:

```
"Generate some epic battle music for the game"
```

Claude will:
1. Use the `generate_music` MCP tool
2. Call ACE-Step to generate the audio
3. Save it to an appropriate location
4. Optionally use `play_music` to play it in the game

## Example Workflow

**User (to Claude):**
> "I need background music for a spooky forest level. Make it atmospheric and creepy."

**Claude's Actions:**
1. Calls `generate_music` tool with:
   - Prompt: "Dark atmospheric forest ambience, mysterious, eerie, subtle wind sounds"
   - Duration: "long" (60s for background music)
   - Style: "ambient"
   - Output: "assets/music/forest_ambient.wav"

2. Uses WebFetch to call ACE-Step:
   ```
   POST http://localhost:7865/generate
   {
     "prompt": "Dark atmospheric forest ambience...",
     "duration": 60,
     "style": "ambient",
     "inference_steps": 27
   }
   ```

3. Receives audio data from ACE-Step

4. Saves to `assets/music/forest_ambient.wav`

5. Calls `play_music`:
   - file_path: "assets/music/forest_ambient.wav"
   - loop: true
   - volume: 0.6 (quieter for ambience)

6. Confirms to user: "I've generated spooky forest music and started playing it in the game!"

## Advanced Usage

### Dynamic Music Based on Game State

```
User: "Make the music adapt to combat intensity"

Claude's approach:
1. Generate multiple tracks:
   - "Tense atmospheric exploration music" → exploration.wav
   - "Moderate combat music with percussion" → combat_light.wav
   - "Intense epic battle music" → combat_intense.wav

2. Use game state to switch tracks:
   - Call play_music with fade_in during transitions
   - Crossfade by overlapping stop_music and play_music
```

### Procedural Soundtracks

```
User: "Each level should have unique music based on its ID"

Claude can:
1. Generate with seed = level_id for reproducibility
2. Vary style based on level theme
3. Cache generated music for faster loading
```

## Integration with Game Code

While Claude handles music generation through MCP, game code can also trigger it programmatically by sending MCP requests.

**Example (Rust):**
```rust
// In game code - send MCP request
let mcp_request = json!({
    "jsonrpc": "2.0",
    "id": 123,
    "method": "tools/call",
    "params": {
        "name": "generate_music",
        "arguments": {
            "prompt": format!("Level {} music", level_id),
            "output_path": format!("assets/music/level_{}.wav", level_id),
            "duration": "extended",
            "style": "electronic"
        }
    }
});

// Send to MCP server via stdin/stdout
// (Implementation depends on how your game communicates with MCP)
```

## Comparison: MCP vs Direct API

### Direct API Approach (engine-ai-music crate):
```rust
// Game code directly calls ACE-Step
let client = AceStepClient::new();
let music = client.generate_music(request).await?;
std::fs::write("battle.wav", &music.audio_data)?;
```

**Pros:** Simple, direct control, typesafe
**Cons:** Tight coupling, requires ACE-Step running locally

### MCP Approach (Current):
```rust
// Game sends MCP request
send_mcp_request("generate_music", args)?;

// Claude handles:
// - Calling ACE-Step (or alternative services)
// - Error handling and retries
// - Optimizing prompts based on context
// - Saving files
```

**Pros:**
- Flexible (Claude can use any music service)
- Intelligent prompt optimization
- No tight coupling to specific services
- Centralized AI operations

**Cons:**
- Requires MCP server running
- Less direct control over API details

## Performance Considerations

- **Generation Time**: 1-30 seconds depending on hardware (see ACE-Step docs)
- **Latency**: Add ~100-500ms for MCP communication overhead
- **Best Practice**: Generate music during loading screens or asynchronously
- **Caching**: Save generated music to disk for reuse

## Troubleshooting

**"Music generation failed"**
- Ensure ACE-Step is running: `curl http://localhost:7865/health`
- Check MCP server logs for errors
- Verify output_path directory exists

**"Slow generation"**
- Check ACE-Step hardware (GPU recommended)
- Reduce duration or inference steps
- Use "fast" quality setting

**"MCP server not responding"**
- Check if server is running: `ps aux | grep engine-mcp-server`
- Verify file-based IPC files in `/tmp/`
- Check logs: `RUST_LOG=debug cargo run --bin engine-mcp-server`

## Future Enhancements

- [ ] Music transitions and crossfading
- [ ] Dynamic parameter adjustment (tempo, intensity)
- [ ] Music variation system (generate variations of existing tracks)
- [ ] Real-time audio analysis for adaptive music
- [ ] Multi-track layering (separate drums, melody, etc.)
- [ ] Support for alternative music generation services (MusicGen, AudioLDM, etc.)

## Resources

- [MCP Specification](https://github.com/anthropics/mcp)
- [ACE-Step Documentation](https://github.com/ace-step/ACE-Step)
- [Game Engine MCP Tools](crates/engine-mcp-server/src/tools.rs)
- [ACE-Step Integration Guide](ACE_STEP_INTEGRATION.md)
