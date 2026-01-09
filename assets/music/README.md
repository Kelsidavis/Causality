# Background Music Directory

Place your music files (.ogg recommended) here.

## Recommended Sources

### Royalty-Free Music
- [Incompetech.com](https://incompetech.com) - Kevin MacLeod's extensive music library
- [OpenGameArt.org](https://opengameart.org/art-search-advanced?keys=&field_art_type_tid%5B%5D=12) - Game music
- [FreePD.com](https://freepd.com) - Public domain music
- [Bensound.com](https://www.bensound.com) - Royalty-free tracks

## Format Recommendations

- **OGG Vorbis**: Best for music (good compression, seamless looping)
- **MP3**: Acceptable but higher CPU overhead
- **WAV**: Avoid for music (too large)

## Usage in Scripts

```rhai
// Start background music
play_music("music/background.ogg", 0.5, true);

// Switch music
stop_music();
play_music("music/battle.ogg", 0.7, true);
```

See `docs/AUDIO_SYSTEM.md` for complete documentation.
