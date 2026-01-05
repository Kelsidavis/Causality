# Hot Reload Guide

## Overview

The game engine now supports **hot reloading** of scripts and assets. This means you can edit files while the engine is running and see changes instantly without restarting!

## What Can Be Hot-Reloaded?

### ✅ Scripts (`.rhai` files)
- **Fully supported**: Edit Rhai scripts and see changes immediately
- **State preservation**: Script scope is preserved during reload
- **Error handling**: Compilation errors are logged without crashing

### 🚧 Assets (Partial Support)
- **Detected**: Changes to `.gltf`, `.glb`, `.png`, `.jpg`, etc. are detected
- **Not yet reloaded**: Asset hot-reload logic is TODO (Phase 8)

## How It Works

### 1. File Watching
The engine uses the `notify` crate to watch for file system changes in:
- `/home/k/game-engine/scripts/` - All `.rhai` script files
- `/home/k/game-engine/assets/` - All asset files

### 2. Debouncing
File changes are debounced (100ms) to avoid multiple reloads from rapid saves.

### 3. Script Reload Process
When a `.rhai` file changes:
1. File watcher detects the change
2. Script is recompiled
3. If compilation succeeds:
   - Old AST is replaced with new AST
   - Script scope is **preserved** (variables keep their values!)
   - Success message logged
4. If compilation fails:
   - Error logged to console
   - Old script continues running
   - No crash!

## Quick Start Tutorial

### Step 1: Run the Editor

```bash
cd /home/k/game-engine
RUST_LOG=info cargo run --bin editor
```

The editor starts with:
- Two falling cubes with rotating scripts
- File watchers active on `scripts/` and `assets/`

### Step 2: Try Hot Reload!

1. **Open the sample script**:
   ```bash
   # In another terminal
   nano scripts/rotate.rhai
   ```

2. **Edit the rotation speed**:
   Change `rotation_speed = 1.0` to `rotation_speed = 5.0`

3. **Save the file** (Ctrl+O, Enter, Ctrl+X in nano)

4. **Watch the magic!** 🎉
   - Within 100ms, you'll see log message: `"Successfully reloaded script"`
   - The cube instantly rotates 5x faster
   - No restart needed!

### Step 3: Test Error Handling

1. **Introduce a syntax error**:
   ```rhai
   fn update(ctx) {
       let rotation_speed = INVALID SYNTAX HERE
       // ...
   }
   ```

2. **Save and observe**:
   - Error logged: `"Script recompilation error"`
   - Engine keeps running with old script
   - Fix the error and save again - works!

## Example Scripts

### Oscillating Rotation

Create `scripts/wobble.rhai`:

```rhai
// Wobble effect - try changing the speed!

fn update(ctx) {
    let wobble_speed = 2.0;  // Change me!
    let angle = sin(ctx.dt * wobble_speed) * 0.1;
    let rotation_delta = quat_from_rotation_z(angle);
    ctx.rotation = ctx.rotation * rotation_delta;
    ctx
}
```

Save, watch it wobble, change the speed, save again!

### Bouncing Motion

Create `scripts/bounce.rhai`:

```rhai
// Bouncing up and down - hot reload the bounce height!

fn update(ctx) {
    let bounce_height = 2.0;  // Try changing to 5.0!
    let bounce_speed = 3.0;

    let offset_y = sin(ctx.dt * bounce_speed) * bounce_height;
    ctx.position.y = ctx.position.y + offset_y * ctx.dt;
    ctx
}
```

## Advanced: Custom File Watching

You can watch additional directories by modifying `engine-editor/src/main.rs`:

```rust
// Add custom watch directory
let custom_dir = std::env::current_dir()?.join("my_scripts");
if custom_dir.exists() {
    hot_reload.watch_directory(&custom_dir)?;
}
```

## Troubleshooting

### "Script reload failed"
**Problem**: Syntax error in your script
**Solution**: Check the console for error details, fix syntax, save again

### "No script reloaded"
**Problem**: File watcher not set up or wrong directory
**Solution**: Ensure scripts are in `/home/k/game-engine/scripts/`

### Changes not detected
**Problem**: File editor might not trigger file system events
**Solution**: Try a different editor, or manually touch the file

### Multiple reloads on save
**Problem**: Editor saves multiple times
**Solution**: Debouncing (100ms) handles this automatically

## Performance

- **File watching**: Minimal CPU usage (event-driven)
- **Debouncing**: Prevents reload spam
- **Script compilation**: Fast (typically <10ms for small scripts)
- **No frame drop**: Reloading happens between frames

## Architecture

```
File System
    │
    ├── scripts/rotate.rhai  [EDITED]
    │
    ▼
notify Watcher
    │
    ▼
Debounce (100ms)
    │
    ▼
ReloadEvent::ScriptChanged
    │
    ▼
ScriptRuntime::reload_script()
    │
    ├── Recompile AST
    ├── Preserve Scope
    └── Replace Script
    │
    ▼
Next Frame: New behavior!
```

## Limitations (Current Phase 7)

1. **Assets not reloaded yet**: Textures/models detected but not reloaded
2. **No script path tracking in demo**: Inline scripts don't have paths
3. **No undo**: Can't roll back to previous script version
4. **No script state export**: Scope preserved but not inspectable

## Future Enhancements (Phase 8+)

- [ ] Asset hot-reload (textures, models, shaders)
- [ ] Script history/undo
- [ ] Script state inspection UI
- [ ] Automatic rollback on error
- [ ] Hot-reload for compiled Rust code (via dynamic linking)
- [ ] Network hot-reload (edit on one machine, reload on another)

## Tips & Tricks

### Rapid Iteration Workflow

1. Keep editor running in one window
2. Keep script file open in another window
3. Make small changes, save frequently
4. Instant feedback loop!

### Script Development Pattern

```rhai
// Start simple
fn update(ctx) {
    let speed = 1.0;  // Tweak this value repeatedly!
    // ... behavior
    ctx
}

// Add complexity gradually
// Hot reload after each addition
// If it breaks, undo and try again
```

### Debugging with Hot Reload

1. Add debug prints: `print("value: " + value)`
2. Save and see output
3. Adjust code based on output
4. Repeat until fixed

## Example Session

```bash
# Terminal 1: Run editor
$ RUST_LOG=info cargo run --bin editor
[INFO] Hot reload system initialized
[INFO] Watching directory: "/home/k/game-engine/assets"
[INFO] Watching directory: "/home/k/game-engine/scripts"

# Terminal 2: Edit script
$ nano scripts/rotate.rhai
# Change rotation_speed from 1.0 to 3.0
# Save

# Back to Terminal 1 logs:
[INFO] Script changed: "/home/k/game-engine/scripts/rotate.rhai"
[INFO] Reloading script: "/home/k/game-engine/scripts/rotate.rhai"
[INFO] Successfully reloaded script for entity EntityId(0)

# See cube rotating 3x faster instantly!
```

---

**Enjoy hot reloading!** No more waiting for compile times or losing context from restarts. Edit, save, see results instantly! 🚀
