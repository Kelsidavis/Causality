// Example: First-Person Character Controller
// Shows how to use CharacterController component with input

fn start(ctx) {
    // Initialize character controller settings
    ctx.move_speed = 7.0;
    ctx.jump_force = 12.0;
    ctx.grounded = true;
    ctx
}

fn update(ctx) {
    // Get input (in a real game, this would come from input system)
    let move_direction = vec3(0.0, 0.0, 0.0);

    // Example: move forward constantly for demo
    move_direction.z = -1.0;

    // Calculate movement
    let speed = ctx.move_speed;
    let movement = move_direction * speed * ctx.dt;

    // Apply movement
    ctx.position = ctx.position + movement;

    // Simple ground check (in real game, use raycasting)
    if ctx.position.y <= 0.0 {
        ctx.position.y = 0.0;
        ctx.grounded = true;
    } else {
        ctx.grounded = false;
    }

    // Jump example (in real game, triggered by input)
    // if input.key_pressed("Space") && ctx.grounded {
    //     ctx.velocity.y = ctx.jump_force;
    // }

    ctx
}
