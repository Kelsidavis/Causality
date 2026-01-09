# Sound Effects Directory

Place your sound effect files (.wav, .ogg, .mp3) here.

## Recommended Sources

### Free Sound Effects
- [Freesound.org](https://freesound.org) - Large collection of CC-licensed sounds
- [OpenGameArt.org](https://opengameart.org/art-search-advanced?keys=&field_art_type_tid%5B%5D=13) - Game-focused audio
- [Zapsplat.com](https://www.zapsplat.com) - Free sound effects library
- [Sonniss GameAudioGDC](https://sonniss.com/gameaudiogdc) - Annual free packs

## Format Recommendations

- **WAV**: Best for short sounds (<2 seconds) - footsteps, gunshots, UI clicks
- **OGG**: Best for longer sounds (>2 seconds) - ambient loops, explosions
- **MP3**: Avoid if possible (higher CPU overhead)

## Example Sounds to Add

```
sounds/
├── footstep.wav        # Player movement
├── jump.wav            # Jump action
├── impact.ogg          # Collision sound
├── button_click.wav    # UI interaction
├── ambient_wind.ogg    # Looping ambient
└── explosion.ogg       # Large event sound
```

## Usage in Scripts

```rhai
// Play a sound effect
play_sound("sounds/jump.wav", 1.0);
```

## Usage in Scenes

```ron
(
    type: "AudioSource",
    audio_path: "sounds/ambient_wind.ogg",
    volume: 0.5,
    max_distance: 50.0,
    looping: true,
    play_on_start: true,
)
```

See `docs/AUDIO_SYSTEM.md` for complete documentation.
