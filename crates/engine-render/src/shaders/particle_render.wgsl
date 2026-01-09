// Particle Rendering Shader
// Renders particles as camera-facing billboards

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

struct CameraUniforms {
    view_proj: mat4x4<f32>,
    camera_pos: vec3<f32>,
    _padding: f32,
    camera_right: vec3<f32>,
    _padding2: f32,
    camera_up: vec3<f32>,
    _padding3: f32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
}

@group(0) @binding(0) var<uniform> camera: CameraUniforms;
@group(1) @binding(0) var particle_texture: texture_2d<f32>;
@group(1) @binding(1) var particle_sampler: sampler;

// Quad vertices for billboard (triangle strip)
const QUAD_VERTICES = array<vec2<f32>, 6>(
    vec2<f32>(-0.5, -0.5),  // Bottom-left
    vec2<f32>( 0.5, -0.5),  // Bottom-right
    vec2<f32>(-0.5,  0.5),  // Top-left
    vec2<f32>(-0.5,  0.5),  // Top-left
    vec2<f32>( 0.5, -0.5),  // Bottom-right
    vec2<f32>( 0.5,  0.5),  // Top-right
);

// UV coordinates for quad
const QUAD_UVS = array<vec2<f32>, 6>(
    vec2<f32>(0.0, 1.0),  // Bottom-left
    vec2<f32>(1.0, 1.0),  // Bottom-right
    vec2<f32>(0.0, 0.0),  // Top-left
    vec2<f32>(0.0, 0.0),  // Top-left
    vec2<f32>(1.0, 1.0),  // Bottom-right
    vec2<f32>(1.0, 0.0),  // Top-right
);

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_index: u32,
    @location(0) particle_position: vec3<f32>,
    @location(1) particle_velocity: vec3<f32>,
    @location(2) particle_color: vec4<f32>,
    @location(3) particle_size: f32,
    @location(4) particle_lifetime: f32,
    @location(5) particle_max_lifetime: f32,
    @location(6) particle_rotation: f32,
) -> VertexOutput {
    var output: VertexOutput;

    // Skip dead particles (moved offscreen by compute shader)
    if (particle_position.y < -9000.0) {
        // Move to far outside clip space
        output.clip_position = vec4<f32>(0.0, 0.0, -100.0, 1.0);
        output.uv = vec2<f32>(0.0, 0.0);
        output.color = vec4<f32>(0.0, 0.0, 0.0, 0.0);
        return output;
    }

    // Get quad vertex position and UV
    let quad_pos = QUAD_VERTICES[vertex_index];
    let uv = QUAD_UVS[vertex_index];

    // Apply rotation
    let cos_r = cos(particle_rotation);
    let sin_r = sin(particle_rotation);
    let rotated = vec2<f32>(
        quad_pos.x * cos_r - quad_pos.y * sin_r,
        quad_pos.x * sin_r + quad_pos.y * cos_r,
    );

    // Billboard: construct world position using camera right/up vectors
    let world_pos = particle_position
        + camera.camera_right * rotated.x * particle_size
        + camera.camera_up * rotated.y * particle_size;

    // Transform to clip space
    output.clip_position = camera.view_proj * vec4<f32>(world_pos, 1.0);
    output.uv = uv;
    output.color = particle_color;

    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Sample particle texture
    let tex_color = textureSample(particle_texture, particle_sampler, input.uv);

    // Apply particle color tint
    var final_color = tex_color * input.color;

    // Discard fully transparent pixels (optimization)
    if (final_color.a < 0.01) {
        discard;
    }

    return final_color;
}
