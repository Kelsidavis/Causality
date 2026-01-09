// Particle Update Compute Shader
// Updates particle physics and lifetime on GPU

struct Particle {
    position: vec3<f32>,
    _padding1: f32,
    velocity: vec3<f32>,
    _padding2: f32,
    color: vec4<f32>,
    size: f32,
    lifetime: f32,
    max_lifetime: f32,
    rotation: f32,
}

struct SimulationUniforms {
    delta_time: f32,
    time: f32,
    _padding1: vec2<f32>,
    gravity: vec3<f32>,
    _padding2: f32,
}

@group(0) @binding(0) var<storage, read_write> particles: array<Particle>;
@group(0) @binding(1) var<uniform> uniforms: SimulationUniforms;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;

    // Bounds check
    if (index >= arrayLength(&particles)) {
        return;
    }

    var particle = particles[index];

    // Update lifetime
    particle.lifetime += uniforms.delta_time;

    // Check if particle is dead
    if (particle.lifetime >= particle.max_lifetime) {
        // Keep particle offscreen if dead
        if (particle.position.y < -9000.0) {
            return;  // Already dead and hidden
        }
        // Kill particle (move offscreen)
        particle.position = vec3<f32>(0.0, -9999.0, 0.0);
        particles[index] = particle;
        return;
    }

    // Apply gravity to velocity
    particle.velocity += uniforms.gravity * uniforms.delta_time;

    // Update position with velocity
    particle.position += particle.velocity * uniforms.delta_time;

    // Calculate life ratio (0.0 = just born, 1.0 = about to die)
    let life_ratio = clamp(particle.lifetime / particle.max_lifetime, 0.0, 1.0);

    // Update size over lifetime (simple linear shrink)
    // TODO: Use curves from emitter properties
    particle.size = mix(1.0, 0.1, life_ratio);

    // Update alpha over lifetime (fade out)
    particle.color.a = 1.0 - life_ratio;

    // Update rotation (simple linear rotation)
    particle.rotation += uniforms.delta_time * 2.0;  // 2 radians per second

    // Write back to buffer
    particles[index] = particle;
}
