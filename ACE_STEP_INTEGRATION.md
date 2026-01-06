# ACE-Step AI Music Generation Integration

This game engine now includes integration with [ACE-Step](https://github.com/ace-step/ACE-Step), a state-of-the-art AI music generation model that can create high-quality music from text descriptions in real-time.

## Overview

ACE-Step is a foundation model for music generation that:
- Generates up to 4 minutes of music in ~20 seconds on an A100 GPU
- 15× faster than LLM-based baselines
- Supports 19 languages for text prompts
- Provides real-time generation on consumer hardware (RTX 4090: 34.48× real-time)
- Supports variations, extensions, and lyric editing

## Architecture

The integration consists of:

1. **ACE-Step Service** (Python) - Runs separately, provides HTTP API
2. **MCP Server** (Rust) - Model Context Protocol server
3. **Claude/AI** - Handles music generation requests
4. **Game Engine** - Sends requests via MCP

```
┌──────────────┐    MCP      ┌────────────┐   HTTP   ┌──────────────┐
│ Game Engine  │────────────►│ MCP Server │─────────►│   Claude     │
│              │             │            │          │              │
│              │             │ 17 Tools   │          │ WebFetch/AI  │
│              │             │  including │          │              │
│              │             │  generate_ │          │              │
│              │             │   music    │          │              │
└──────────────┘             └────────────┘          └──────┬───────┘
                                                            │
                                                            │ HTTP
                                                            ▼
                                                     ┌─────────────┐
                                                     │  ACE-Step   │
                                                     │  API Server │
                                                     │ (localhost) │
                                                     └─────────────┘
```

**Recommended Approach (MCP):**
- Game engine sends MCP request for music
- Claude receives request through MCP server
- Claude uses WebFetch to call ACE-Step API
- Claude saves generated audio and confirms to game

**Alternative (Direct):**
- Use `engine-ai-music` crate for direct API calls
- More control but tighter coupling

See [MCP_MUSIC_GENERATION.md](MCP_MUSIC_GENERATION.md) for the MCP approach.

## Quick Start

### 1. Setup ACE-Step Service

The ACE-Step repository has been cloned to `/home/k/game-engine/ACE-Step`.

To run the service:

```bash
cd ACE-Step
source venv/bin/activate

# Basic usage (CPU)
acestep --port 7865

# With GPU (recommended)
acestep --port 7865 --device_id 0

# With optimizations
acestep --port 7865 --device_id 0 --bf16 true --torch_compile true
```

The service will:
- Download the model (~2GB) on first run
- Start an HTTP server on port 7865
- Be ready to receive music generation requests

### 2. Use in Game Engine

```rust
use engine_ai_music::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create client
    let client = AceStepClient::new();

    // Generate music
    let request = MusicGenerationRequest::new("Epic battle music")
        .with_duration(MusicDuration::Medium)
        .with_style(MusicStyle::Cinematic)
        .instrumental();

    let music = client.generate_music(request).await?;

    // Save or play the generated audio
    std::fs::write("battle.wav", &music.audio_data)?;

    Ok(())
}
```

## Use Cases

### Dynamic Music Generation

Generate music on-the-fly based on game state:

```rust
// Boss battle starts
let battle_music = client.generate_music(
    MusicGenerationRequest::new("Intense boss battle with heavy drums")
        .with_style(MusicStyle::Metal)
        .with_tempo(160)
).await?;
```

### Procedural Soundtracks

Create unique music for each playthrough:

```rust
// Generate level music with a seed based on level ID
let level_music = client.generate_music(
    MusicGenerationRequest::new("Mysterious dungeon ambience")
        .with_seed(level_id as u64)
        .with_duration(MusicDuration::Extended)
).await?;
```

### Music Variations

Create variations of existing themes:

```rust
let original_theme = std::fs::read("main_theme.wav")?;
let variation = client.generate_variation(&original_theme, 0.3).await?;
```

### Adaptive Music

Extend music seamlessly during gameplay:

```rust
// Player is taking longer than expected
let extended = client.extend_audio(&current_music, false, 60).await?;
```

## Integration with Audio System

The AI music generation works seamlessly with the existing audio system:

```rust
use engine_audio::AudioSystem;
use engine_ai_music::prelude::*;

async fn play_dynamic_music(
    audio_system: &mut AudioSystem,
    ai_client: &AceStepClient,
    game_state: &GameState,
) -> Result<(), Box<dyn std::error::Error>> {
    // Generate music based on game state
    let prompt = match game_state.phase {
        GamePhase::Exploration => "Calm exploration music",
        GamePhase::Combat => "Intense combat music",
        GamePhase::Victory => "Triumphant victory fanfare",
    };

    let music = ai_client.generate_music(
        MusicGenerationRequest::new(prompt)
            .with_duration(MusicDuration::Long)
    ).await?;

    // Save temporarily
    std::fs::write("temp_music.wav", &music.audio_data)?;

    // Play through audio system
    audio_system.play_music("temp_music.wav", true);

    Ok(())
}
```

## Performance Characteristics

### Generation Times (27 inference steps)

| Hardware          | Speed (Real-time Factor) | Time per Minute |
|-------------------|--------------------------|-----------------|
| RTX 4090          | 34.48×                   | 1.74s           |
| RTX 3080          | ~20×                     | ~3s             |
| MacBook M2 Max    | 2.27×                    | 26.43s          |
| CPU (AVX2)        | ~0.1×                    | ~600s           |

### Resource Requirements

- **GPU VRAM**: 4-8 GB (use `--cpu_offload` for 8GB systems)
- **System RAM**: ~4 GB
- **Disk**: ~2 GB for model files
- **Network**: Local (localhost) or LAN/cloud

### Optimization Tips

1. **Pre-generation**: Generate music during loading screens
2. **Caching**: Cache generated music with seed values for reproducibility
3. **Quality vs Speed**: Reduce inference steps (15-20) for faster generation
4. **Duration**: Generate shorter clips and extend/loop as needed
5. **Remote Service**: Run ACE-Step on a dedicated server/cloud instance

## Production Deployment

### Separate Server Approach

For production games, run ACE-Step on a separate machine:

```rust
let config = AceStepConfig::with_url("http://music-generator.example.com:7865")
    .with_timeout(600);

let client = AceStepClient::with_config(config);
```

### Docker Deployment

ACE-Step includes a Dockerfile for containerized deployment:

```bash
docker compose up
```

### Cloud Deployment

- Deploy to GPU-enabled cloud instances (AWS, GCP, Azure)
- Use load balancers for multiple instances
- Cache generated music in cloud storage

## API Reference

See `crates/engine-ai-music/README.md` for complete API documentation.

Key components:
- `AceStepClient` - Main client for API calls
- `MusicGenerationRequest` - Configuration for music generation
- `MusicDuration` - Duration presets (Short/Medium/Long/Extended)
- `MusicStyle` - Genre presets (Rock/Pop/Electronic/Jazz/etc.)
- `GenerationResult` - Generated audio data and metadata

## Examples

Run the example:

```bash
cargo run --example generate_music -p engine-ai-music
```

This will generate three music tracks demonstrating different styles.

## Limitations

1. **Resource Intensive**: Requires GPU for real-time performance
2. **Quality Variability**: AI-generated music may not always match expectations
3. **Latency**: Even on fast hardware, generation takes 1-2 seconds minimum
4. **Network Dependency**: Requires ACE-Step service to be running
5. **Determinism**: Same seed produces same output, but prompt variations can differ

## Future Enhancements

- [ ] Music transition system (crossfade between generated tracks)
- [ ] Style transfer (apply style of one track to another)
- [ ] Real-time parameter adjustment
- [ ] Integration with game event system
- [ ] Music caching and asset management
- [ ] Offline mode with pre-generated music library

## License

- **ACE-Step**: Apache 2.0 License
- **Integration Layer**: Part of this game engine

## Resources

- [ACE-Step GitHub](https://github.com/ace-step/ACE-Step)
- [ACE-Step Paper](https://arxiv.org/abs/XXXX.XXXXX) (if available)
- [Engine AI Music Crate](crates/engine-ai-music/)
- [Example Code](crates/engine-ai-music/examples/)

## Support

For issues with:
- **ACE-Step itself**: Report to https://github.com/ace-step/ACE-Step/issues
- **Integration layer**: Report to this game engine's issue tracker
