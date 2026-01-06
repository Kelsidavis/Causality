# AI-Generated Game Music Collection

This collection of music tracks was generated using **ACE-Step**, an AI music generation model, integrated with the game engine through the MCP (Model Context Protocol) system.

## Generated Tracks

### Main Theme
- **sunny_exploration.wav** (11 MB)
  - **Mood**: Uplifting, cheerful, peaceful
  - **Description**: Light acoustic guitar, soft piano melody, instrumental
  - **Use Case**: Open-world exploration, sunny environments, peaceful gameplay
  - **Duration**: ~60 seconds

### Sample Collection (`samples/`)

#### 1. battle.wav (11 MB)
- **Mood**: Epic, intense, heroic
- **Description**: Orchestral with heavy drums and brass, powerful percussion, heroic fanfare
- **Use Case**: Combat sequences, general battles, action gameplay
- **Duration**: ~60 seconds

#### 2. boss_fight.wav (11 MB)
- **Mood**: Intense, dramatic, aggressive
- **Description**: Dramatic orchestral with heavy percussion, epic battle theme
- **Use Case**: Boss encounters, climactic battles, major confrontations
- **Duration**: ~60 seconds

#### 3. calm.wav (11 MB)
- **Mood**: Peaceful, serene, meditative
- **Description**: Soft ambient pads, gentle flowing water sounds, tranquil atmosphere
- **Use Case**: Safe zones, meditation/rest areas, peaceful villages, menu screens
- **Duration**: ~60 seconds

#### 4. dungeon.wav (11 MB)
- **Mood**: Dark, mysterious, ominous
- **Description**: Atmospheric ambient with deep bass, mysterious cave sounds
- **Use Case**: Underground areas, dungeons, dark caves, dangerous zones
- **Duration**: ~60 seconds

#### 5. mysterious.wav (11 MB)
- **Mood**: Eerie, suspenseful, enigmatic
- **Description**: Dark ambient soundscape, suspenseful atmosphere
- **Use Case**: Puzzle areas, mysterious locations, investigation sequences
- **Duration**: ~60 seconds

#### 6. victory.wav (11 MB)
- **Mood**: Triumphant, celebratory, joyful
- **Description**: Upbeat celebration, heroic fanfare, joyful orchestral
- **Use Case**: Victory screens, quest completions, achievements unlocked
- **Duration**: ~60 seconds

## Technical Details

**Generation Method**: ACE-Step AI Model (v1-3.5B)
**Format**: WAV (16-bit stereo, 48 kHz)
**Quality**: Professional studio quality
**All Tracks**: 100% instrumental, no vocals
**Total Size**: ~77 MB (7 tracks)

## Generation Parameters

- **Model**: ACE-Step v1-3.5B parameters
- **Inference Steps**: 27 (balanced quality/speed)
- **Guidance Scale**: 3.5
- **Duration**: 60 seconds each
- **Hardware**: CPU generation (Intel/AMD)
- **Generation Time**: ~2 minutes per track

## How to Use

### In Game Code

```rust
use engine_audio::AudioSystem;

// Play background music
audio_system.play_music("assets/music/samples/battle.wav", true);

// Stop music with fade out
audio_system.stop_music();
```

### Through MCP (Recommended)

Use the MCP tools to control music playback:

```json
{
  "method": "tools/call",
  "params": {
    "name": "play_music",
    "arguments": {
      "file_path": "assets/music/samples/battle.wav",
      "loop": true,
      "volume": 0.8,
      "fade_in": 2.0
    }
  }
}
```

## Regeneration

To generate new music with different moods, use the ACE-Step integration:

```bash
cd ACE-Step
source venv/bin/activate
python generate_music_simple.py "your prompt here" output.wav
```

Or ask Claude through MCP:
> "Generate some spooky horror music for my game"

## Licensing

These tracks were generated using the ACE-Step AI model:
- **Model**: Apache 2.0 License
- **Generated Content**: Check ACE-Step licensing for AI-generated music rights
- **Use**: Intended for game development and testing

## Customization

Want different moods? Generate more tracks by modifying prompts:
- **Themes**: Sci-fi, fantasy, horror, western, cyberpunk
- **Intensity**: Calm, moderate, intense, extreme
- **Instruments**: Piano, guitar, orchestral, electronic, ambient
- **Tempo**: Slow (60-90 BPM), medium (90-120), fast (120-160+)

## Future Enhancements

- [ ] Music transitions and crossfading between tracks
- [ ] Dynamic music that adapts to gameplay intensity
- [ ] Longer tracks (2-4 minutes)
- [ ] Variations of existing themes
- [ ] Layer-based music system (drums, melody, harmony separately)

## See Also

- [ACE-Step Integration Guide](../../ACE_STEP_INTEGRATION.md)
- [MCP Music Generation Guide](../../MCP_MUSIC_GENERATION.md)
- [Audio System Documentation](../../crates/engine-audio/README.md)

---

Generated on: 2026-01-06
AI Model: ACE-Step v1-3.5B
Integration: Claude Code + MCP Server
