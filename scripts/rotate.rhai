// Rotation script - can be hot-reloaded!
// Try changing the rotation_speed value and save to see instant updates

fn update(ctx) {
    // Rotate around Y axis
    let rotation_speed = 1.0;  // Change this value and save!
    let angle = ctx.dt * rotation_speed;
    let rotation_delta = quat_from_rotation_y(angle);
    ctx.rotation = ctx.rotation * rotation_delta;
    ctx
}
