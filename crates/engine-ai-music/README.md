# Engine AI Music - ACE-Step Integration

AI-powered music generation for the game engine using [ACE-Step](https://github.com/ace-step/ACE-Step), a fast foundation model for music generation.

## Features

- **Text-to-Music Generation**: Generate music from text descriptions
- **Style Control**: Support for multiple music genres (rock, electronic, jazz, classical, etc.)
- **Duration Control**: Generate music of various lengths (15s to 2+ minutes)
- **Variations**: Create variations of existing audio
- **Audio Extension**: Extend music by adding segments before/after
- **Async API**: Non-blocking music generation using Tokio

## Setup

### 1. Install ACE-Step

First, clone and set up ACE-Step:

```bash
cd /path/to/your/projects
git clone https://github.com/ace-step/ACE-Step.git
cd ACE-Step
python3 -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate
pip install -e .
```

### 2. Run ACE-Step API Server

Start the ACE-Step service:

```bash
# Basic usage (CPU)
acestep --port 7865

# With GPU (recommended for faster generation)
acestep --port 7865 --device_id 0

# With optimizations for low VRAM systems
acestep --port 7865 --cpu_offload --overlapped_decode

# Full performance (requires triton on Windows)
acestep --port 7865 --device_id 0 --bf16 true --torch_compile true
```

The service will download the model on first run (~2GB).

### 3. Use in Game Engine

```rust
use engine_ai_music::prelude::*;

#[tokio::main]
async fn main() -> Result<(), AceStepError> {
    // Create client
    let client = AceStepClient::new();

    // Check if service is running
    let health = client.health_check().await?;
    println!("ACE-Step ready: {:?}", health);

    // Generate music
    let request = MusicGenerationRequest::new("Epic battle music with drums")
        .with_duration(MusicDuration::Medium)
        .with_style(MusicStyle::Cinematic)
        .instrumental();

    let result = client.generate_music(request).await?;
    println!("Generated {} bytes of audio", result.audio_data.len());

    // Save to file
    std::fs::write("battle_music.wav", &result.audio_data)?;

    Ok(())
}
```

## Usage Examples

### Basic Music Generation

```rust
let client = AceStepClient::new();

let request = MusicGenerationRequest::new("Upbeat electronic dance music")
    .with_duration(MusicDuration::Long)
    .with_style(MusicStyle::Electronic)
    .with_tempo(128)
    .instrumental();

let music = client.generate_music(request).await?;
```

### Generate and Save

```rust
let request = MusicGenerationRequest::new("Calm ambient meditation music")
    .with_style(MusicStyle::Ambient)
    .with_duration(MusicDuration::Extended);

client.generate_and_save(request, "ambient.wav").await?;
```

### Create Variations

```rust
// Load existing audio
let original = std::fs::read("music.wav")?;

// Generate variation (0.5 = moderate variation)
let variation = client.generate_variation(&original, 0.5).await?;
```

### Extend Audio

```rust
let original = std::fs::read("short_clip.wav")?;

// Extend by 30 seconds after the clip
let extended = client.extend_audio(&original, false, 30).await?;
```

### Custom Configuration

```rust
let config = AceStepConfig::with_url("http://my-server:8000")
    .with_timeout(600); // 10 minutes

let client = AceStepClient::with_config(config);
```

## Integration with Audio System

Combine with the game engine's audio system:

```rust
use engine_ai_music::prelude::*;
use engine_audio::AudioSystem;

async fn generate_dynamic_music(
    ai_client: &AceStepClient,
    audio_system: &mut AudioSystem,
) -> Result<(), AceStepError> {
    // Generate music based on game state
    let request = MusicGenerationRequest::new("Intense boss battle music")
        .with_style(MusicStyle::Metal)
        .with_tempo(160)
        .instrumental();

    let result = ai_client.generate_music(request).await?;

    // Save temporarily
    std::fs::write("temp_music.wav", &result.audio_data)?;

    // Play through audio system
    audio_system.play_music("temp_music.wav", true);

    Ok(())
}
```

## Performance Considerations

- **Generation Time**: Varies based on hardware and duration
  - RTX 4090: ~1.74s per minute of audio (34.48× real-time)
  - MacBook M2 Max: ~26.43s per minute (2.27× real-time)
  - CPU: Much slower, not recommended for real-time

- **Resource Usage**:
  - GPU VRAM: ~4-8GB (use `--cpu_offload` if limited)
  - CPU RAM: ~4GB
  - Disk: ~2GB for model files

- **Best Practices**:
  - Pre-generate music during loading screens
  - Cache generated music for reuse
  - Run ACE-Step on a separate machine/cloud for production
  - Use shorter durations for faster generation
  - Reduce inference steps (15-20) for speed vs quality tradeoff

## API Reference

### MusicGenerationRequest

- `new(prompt)` - Create request with text description
- `with_duration(duration)` - Set music length
- `with_style(style)` - Set genre/style
- `with_tempo(bpm)` - Set tempo in BPM
- `instrumental()` - Remove vocals
- `with_seed(seed)` - Set random seed for reproducibility
- `with_steps(steps)` - Set inference steps (quality vs speed)

### MusicDuration

- `Short` - ~15 seconds
- `Medium` - ~30 seconds
- `Long` - ~60 seconds
- `Extended` - ~2 minutes
- `Custom(secs)` - Custom duration

### MusicStyle

Predefined: `Rock`, `Pop`, `Electronic`, `Jazz`, `Classical`, `HipHop`, `Ambient`, `Cinematic`, `Folk`, `Metal`, `Indie`

Or use `Custom(String)` for specific descriptions.

## Troubleshooting

**Service not available**
- Ensure ACE-Step server is running: `acestep --port 7865`
- Check the URL in config matches server address

**Timeout errors**
- Increase timeout in config: `.with_timeout(600)`
- Reduce duration or inference steps
- Check GPU availability

**Out of memory**
- Use `--cpu_offload` flag when starting ACE-Step
- Generate shorter clips
- Close other GPU applications

**Poor quality**
- Increase inference steps (default: 27, try 35-50)
- Use more descriptive prompts
- Ensure GPU acceleration is enabled

## License

This integration layer is part of the game engine. ACE-Step itself is licensed under Apache 2.0.
