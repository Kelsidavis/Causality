// Audio Demo Script
// Demonstrates audio playback from scripts

fn start(context) {
    print("Audio demo started!");

    // Play background music (looping)
    // play_music("music/background.ogg", 0.5, true);

    // Note: Uncomment the above line when you have audio files
    // Place audio files in assets/sounds/ and assets/music/
}

fn update(context) {
    let dt = context.dt;
    let pos = context.position;

    // Example: Play sound when entity reaches certain position
    // if pos.y < 0.5 {
    //     play_sound("sounds/impact.wav", 1.0);
    // }

    // Rotate the object
    let rotation = context.rotation;
    let angle = dt * 1.0; // Rotate 1 radian per second
    context.rotation = rotation * quat_from_rotation_y(angle);

    context
}
