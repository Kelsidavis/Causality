# Audio System Guide

## Overview

The Causality Engine includes a complete 3D spatial audio system powered by rodio. It supports:
- **2D Sound Effects** - Non-positional sounds (UI clicks, menu sounds)
- **3D Spatial Audio** - Position-based sound with distance attenuation
- **Background Music** - Looping music tracks
- **Script Integration** - Full audio control from Rhai scripts

## Quick Start

### 1. Adding Audio Assets

Place audio files in the appropriate directories:
```
assets/
├── sounds/          # Sound effects (.wav, .ogg, .mp3)
│   ├── footstep.wav
│   ├── jump.wav
│   └── impact.ogg
└── music/           # Background music (.ogg recommended for loops)
    ├── background.ogg
    └── menu.ogg
```

**Supported Formats:**
- WAV (uncompressed, best for short sounds)
- OGG Vorbis (compressed, best for music)
- MP3 (compressed, widely supported)

**Recommended Sources:**
- [Freesound.org](https://freesound.org) - Free sound effects
- [OpenGameArt.org](https://opengameart.org) - Free game audio
- [Incompetech.com](https://incompetech.com) - Royalty-free music

### 2. Using Audio Components

Add AudioSource components to entities in your scene:

```ron
(
    id: (5),
    name: "Ambient Sound",
    transform: (
        position: (10.0, 0.0, 5.0),
        rotation: (0.0, 0.0, 0.0, 1.0),
        scale: (1.0, 1.0, 1.0),
    ),
    components: [
        (
            type: "AudioSource",
            audio_path: "sounds/ambient.ogg",
            volume: 0.7,
            max_distance: 50.0,
            playing: false,
            looping: true,
            play_on_start: true,
        ),
    ],
)
```

**AudioSource Fields:**
- `audio_path`: Path to audio file (relative to `assets/`)
- `volume`: Volume level (0.0 to 1.0)
- `max_distance`: Maximum hearing distance (in world units)
- `playing`: Whether currently playing (managed by system)
- `looping`: Whether to loop continuously
- `play_on_start`: Auto-play when scene starts

### 3. Script API

Control audio playback from Rhai scripts:

```rust
// Play a 2D sound effect
play_sound("sounds/jump.wav", 1.0);  // path, volume

// Play background music
play_music("music/background.ogg", 0.5, true);  // path, volume, looping

// Stop background music
stop_music();
```

**Example Script:**

```rhai
fn start(context) {
    // Play menu music when entity starts
    play_music("music/menu.ogg", 0.6, true);
}

fn update(context) {
    let pos = context.position;

    // Play sound when entity hits ground
    if pos.y <= 0.0 {
        play_sound("sounds/impact.wav", 0.8);
    }

    context
}
```

## 3D Spatial Audio

### How It Works

The audio system automatically calculates:
1. **Distance Attenuation** - Sounds get quieter with distance
2. **Listener Position** - Audio is relative to camera position
3. **Volume Falloff** - Quadratic falloff based on `max_distance`

### Attenuation Formula

```rust
distance = length(sound_position - listener_position)
attenuation = if distance < max_distance {
    1.0 - (distance / max_distance)²
} else {
    0.0
}
final_volume = base_volume * attenuation
```

### Best Practices

**For Sound Effects:**
- Use WAV for short sounds (<2 seconds)
- Keep max_distance reasonable (10-50 units)
- Use higher volume (0.8-1.0) for important sounds

**For Ambient Sounds:**
- Use OGG for longer loops (>5 seconds)
- Set larger max_distance (50-100 units)
- Use lower volume (0.3-0.7)
- Enable looping and play_on_start

**For Background Music:**
- Use OGG format for best compression
- Control via scripts, not components
- Fade between tracks by adjusting volume

## API Reference

### Script Functions

#### `play_sound(path: str, volume: float) -> bool`

Plays a 2D sound effect (no spatial position).

**Parameters:**
- `path`: Path to audio file (relative to `assets/`)
- `volume`: Volume level (0.0 to 1.0)

**Returns:** `true` on success, `false` on error

**Example:**
```rhai
play_sound("sounds/click.wav", 1.0);
```

---

#### `play_music(path: str, volume: float, looping: bool) -> bool`

Plays background music. Stops any currently playing music.

**Parameters:**
- `path`: Path to music file (relative to `assets/`)
- `volume`: Volume level (0.0 to 1.0)
- `looping`: Whether to loop the track

**Returns:** `true` on success, `false` on error

**Example:**
```rhai
play_music("music/battle.ogg", 0.6, true);
```

---

#### `stop_music()`

Stops the currently playing background music.

**Example:**
```rhai
stop_music();
```

## Scene Components

### AudioSource

Attaches a 3D positional audio source to an entity.

**Fields:**
```ron
(
    type: "AudioSource",
    audio_path: String,      // "sounds/ambient.ogg"
    volume: f32,             // 0.0 to 1.0
    max_distance: f32,       // World units (10.0 - 100.0)
    playing: bool,           // Read-only (managed by system)
    looping: bool,           // true for continuous play
    play_on_start: bool,     // Auto-play when scene loads
)
```

**Example:**
```ron
(
    type: "AudioSource",
    audio_path: "sounds/waterfall.ogg",
    volume: 0.5,
    max_distance: 75.0,
    playing: false,
    looping: true,
    play_on_start: true,
)
```

### AudioListener

Defines where the player hears sounds from (usually attached to camera).

**Note:** Currently, the audio listener is automatically attached to the camera. Future versions may support multiple listeners or manual control.

## Performance Tips

1. **Limit Active Sounds** - Too many simultaneous sounds can impact performance
2. **Use Appropriate Formats**:
   - WAV: Short sounds (<2s)
   - OGG: Music and ambient loops
   - Avoid MP3 if possible (higher decode overhead)
3. **Reasonable Max Distance** - Don't set max_distance too high unnecessarily
4. **Sound Pooling** - Reuse common sounds (footsteps, gunshots)

## Troubleshooting

### "Failed to play audio" Error

**Possible Causes:**
1. Audio file doesn't exist at specified path
2. File is corrupted or in unsupported format
3. Audio output device not available

**Solutions:**
- Verify file path is correct (relative to `assets/`)
- Test audio file in media player
- Check console logs for specific error

### Sound Too Quiet or Inaudible

**Check:**
1. Volume setting (should be 0.5-1.0 for audible sounds)
2. Distance from listener (must be within `max_distance`)
3. Master system volume

**Debug:**
```rhai
// Test with very loud, close sound
play_sound("sounds/test.wav", 1.0);
```

### Music Not Looping

**Ensure:**
- `looping` parameter is set to `true` in `play_music` call
- Audio file has no silence at start/end (causes gaps)

## Example Scenes

### Simple Audio Test

```ron
(
    name: "Audio Test",
    entities: {
        (1): (
            id: (1),
            name: "Music Player",
            transform: (position: (0.0, 0.0, 0.0), rotation: (0.0, 0.0, 0.0, 1.0), scale: (1.0, 1.0, 1.0)),
            components: [
                (
                    type: "Script",
                    source_path: "scripts/audio_demo.rhai",
                    enabled: true,
                ),
            ],
        ),
        (2): (
            id: (2),
            name: "Ambient Sound",
            transform: (position: (10.0, 0.0, 0.0), rotation: (0.0, 0.0, 0.0, 1.0), scale: (1.0, 1.0, 1.0)),
            components: [
                (
                    type: "AudioSource",
                    audio_path: "sounds/ambient.ogg",
                    volume: 0.7,
                    max_distance: 50.0,
                    playing: false,
                    looping: true,
                    play_on_start: true,
                ),
            ],
        ),
    },
)
```

## Integration with Game Systems

### Footstep Sounds (Physics Collision)

```rhai
fn update(context) {
    let pos = context.position;

    // Detect ground collision
    if pos.y <= 0.01 {
        play_sound("sounds/footstep.wav", 0.5);
    }

    context
}
```

### UI Sounds (Input Detection)

```rhai
fn update(context) {
    // Assuming input API is available
    if is_action_just_pressed("select") {
        play_sound("sounds/button_click.wav", 0.8);
    }

    context
}
```

### Dynamic Music (Game State)

```rhai
fn start(context) {
    // Start with calm music
    play_music("music/calm.ogg", 0.5, true);
}

fn update(context) {
    // Switch to battle music on trigger
    if should_switch_to_battle_music() {
        stop_music();
        play_music("music/battle.ogg", 0.7, true);
    }

    context
}
```

## Future Enhancements

Potential additions for future versions:
- **Audio Zones** - Reverb and echo effects in specific areas
- **Doppler Effect** - Pitch shift for moving sounds
- **3D Panning** - Stereo positioning based on direction
- **Audio Triggers** - Play sounds on physics collisions
- **Volume Ducking** - Lower music when SFX plays
- **Audio Mixer** - Separate volume controls (music, SFX, voice)

## Technical Details

### Architecture

```
Scripts → AudioCommandQueue → AudioSystem → rodio → OS Audio
```

**Thread Safety:**
- Scripts push commands to thread-safe queue
- Main thread processes commands each frame
- rodio handles audio output on dedicated thread

**Dependencies:**
- `rodio` - Audio playback library
- `crossbeam-channel` - Thread-safe message passing

### File Format Support

rodio supports:
- WAV (PCM uncompressed)
- Vorbis (OGG container)
- Flac
- MP3 (via minimp3)

**Note:** Format availability depends on rodio features enabled.

---

**Next Steps:**
1. Add audio files to `assets/sounds/` and `assets/music/`
2. Create test scene with AudioSource components
3. Write scripts that use audio API
4. Experiment with spatial audio and distance attenuation

For more information, see:
- `scripts/audio_demo.rhai` - Example script
- `docs/QUICK_REFERENCE.md` - Audio API quick reference
- `assets/scenes/audio_test.ron` - Test scene (if created)
