// Water shader with transparency, waves, and fresnel

struct Uniforms {
    view_proj: mat4x4<f32>,
    camera_pos: vec3<f32>,
    time: f32,
    flow_direction: vec2<f32>,
    flow_speed: f32,
    _padding: vec2<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

// Texture and sampler
@group(1) @binding(0)
var t_texture: texture_2d<f32>;
@group(1) @binding(1)
var t_sampler: sampler;

// Shadow map
@group(2) @binding(0)
var shadow_texture: texture_depth_2d;
@group(2) @binding(1)
var shadow_sampler: sampler_comparison;
@group(2) @binding(2)
var<uniform> shadow_uniforms: ShadowUniforms;

struct ShadowUniforms {
    light_space_matrix: mat4x4<f32>,
}

// Push constants for per-object data (model matrix)
struct PushConstants {
    model: mat4x4<f32>,
}
var<push_constant> push: PushConstants;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tex_coord: vec2<f32>,
    @location(3) color: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tex_coord: vec2<f32>,
    @location(3) color: vec3<f32>,
    @location(4) shadow_position: vec4<f32>,
    @location(5) view_direction: vec3<f32>,
}

// Wave parameters - multiple overlapping waves for natural look
const WAVE_SPEED: f32 = 1.2;
const WAVE_FREQUENCY: f32 = 0.5;
const WAVE_AMPLITUDE: f32 = 0.25;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    // Transform vertex to world space first
    var world_position = push.model * vec4<f32>(in.position, 1.0);

    // Apply multiple overlapping waves in WORLD space for natural variation
    // Waves travel in flow direction
    let flow_dir = vec2<f32>(uniforms.flow_direction.x, uniforms.flow_direction.y);
    let flow_influence = uniforms.flow_speed * 0.5;
    let pos_along_flow = world_position.x * flow_dir.x + world_position.z * flow_dir.y;

    let wave1 = sin((world_position.x + world_position.z) * WAVE_FREQUENCY + uniforms.time * WAVE_SPEED + pos_along_flow * flow_influence) * WAVE_AMPLITUDE;
    let wave2 = sin((world_position.x * 0.7 - world_position.z * 1.3) * WAVE_FREQUENCY * 1.3 + uniforms.time * WAVE_SPEED * 0.9) * WAVE_AMPLITUDE * 0.4;
    let wave3 = sin((world_position.x * 1.5 + world_position.z * 0.5) * WAVE_FREQUENCY * 2.0 + uniforms.time * WAVE_SPEED * 1.4 + pos_along_flow * flow_influence * 0.5) * WAVE_AMPLITUDE * 0.2;
    let wave4 = sin((world_position.x * 0.3 - world_position.z * 2.0) * WAVE_FREQUENCY * 0.7 + uniforms.time * WAVE_SPEED * 0.6) * WAVE_AMPLITUDE * 0.3;
    world_position.y += wave1 + wave2 + wave3 + wave4;

    out.clip_position = uniforms.view_proj * world_position;
    out.world_position = world_position.xyz;

    // Calculate wave normal from combined wave derivatives
    let dx1 = cos((world_position.x + world_position.z) * WAVE_FREQUENCY + uniforms.time * WAVE_SPEED) * WAVE_AMPLITUDE * WAVE_FREQUENCY;
    let dx2 = cos((world_position.x * 0.7 - world_position.z * 1.3) * WAVE_FREQUENCY * 1.3 + uniforms.time * WAVE_SPEED * 0.9) * WAVE_AMPLITUDE * 0.4 * WAVE_FREQUENCY * 1.3 * 0.7;
    let dx3 = cos((world_position.x * 1.5 + world_position.z * 0.5) * WAVE_FREQUENCY * 2.0 + uniforms.time * WAVE_SPEED * 1.4) * WAVE_AMPLITUDE * 0.2 * WAVE_FREQUENCY * 2.0 * 1.5;
    let wave_dx = dx1 + dx2 + dx3;

    let dz1 = cos((world_position.x + world_position.z) * WAVE_FREQUENCY + uniforms.time * WAVE_SPEED) * WAVE_AMPLITUDE * WAVE_FREQUENCY;
    let dz2 = -cos((world_position.x * 0.7 - world_position.z * 1.3) * WAVE_FREQUENCY * 1.3 + uniforms.time * WAVE_SPEED * 0.9) * WAVE_AMPLITUDE * 0.4 * WAVE_FREQUENCY * 1.3 * 1.3;
    let dz3 = cos((world_position.x * 1.5 + world_position.z * 0.5) * WAVE_FREQUENCY * 2.0 + uniforms.time * WAVE_SPEED * 1.4) * WAVE_AMPLITUDE * 0.2 * WAVE_FREQUENCY * 2.0 * 0.5;
    let wave_dz = dz1 + dz2 + dz3;

    var wave_normal = normalize(vec3<f32>(-wave_dx, 1.0, -wave_dz));

    // Transform normal
    let normal_matrix = mat3x3<f32>(
        push.model[0].xyz,
        push.model[1].xyz,
        push.model[2].xyz,
    );
    out.normal = normalize(normal_matrix * wave_normal);

    out.tex_coord = in.tex_coord;
    out.color = in.color;

    // Calculate shadow position
    out.shadow_position = shadow_uniforms.light_space_matrix * world_position;

    // View direction for fresnel
    out.view_direction = normalize(uniforms.camera_pos - world_position.xyz);

    return out;
}

// Calculate shadow (simple, no PCF)
fn calculate_shadow(shadow_pos: vec4<f32>) -> f32 {
    // Perform perspective divide
    let proj_coords = shadow_pos.xyz / shadow_pos.w;

    // Transform from NDC [-1,1] to texture coords [0,1]
    let shadow_uv = vec2<f32>(
        proj_coords.x * 0.5 + 0.5,
        proj_coords.y * -0.5 + 0.5  // Flip Y for texture coordinates
    );
    let shadow_depth = proj_coords.z;  // Already in [0,1] for WebGPU

    // Check if outside shadow map bounds
    if shadow_uv.x < 0.0 || shadow_uv.x > 1.0 ||
       shadow_uv.y < 0.0 || shadow_uv.y > 1.0 ||
       shadow_depth < 0.0 || shadow_depth > 1.0 {
        return 1.0; // Outside shadow map = fully lit
    }

    // Apply bias to prevent shadow acne
    let bias = 0.002;

    // Single sample shadow comparison
    let shadow = textureSampleCompare(
        shadow_texture,
        shadow_sampler,
        shadow_uv,
        shadow_depth - bias
    );

    return shadow;
}

// Fresnel effect (Schlick approximation)
fn fresnel(view_dir: vec3<f32>, normal: vec3<f32>, f0: f32) -> f32 {
    let cos_theta = max(dot(view_dir, normal), 0.0);
    return f0 + (1.0 - f0) * pow(1.0 - cos_theta, 5.0);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Lighting - matches shadow map light direction
    let light_dir = normalize(vec3<f32>(-0.8, 0.5, -0.4));
    let ambient = 0.2;
    let diffuse = max(dot(in.normal, light_dir), 0.0);

    // Calculate shadow
    let shadow = calculate_shadow(in.shadow_position);

    // Apply shadow to diffuse light
    let lighting = ambient + diffuse * 0.8 * shadow;

    // Sample texture with flow-based animated UVs
    let flow_offset = uniforms.flow_direction * uniforms.flow_speed * uniforms.time * 0.1;
    let base_scroll = vec2<f32>(uniforms.time * 0.003, uniforms.time * 0.002);
    let animated_uv = in.tex_coord + flow_offset + base_scroll;
    let tex_color = textureSample(t_texture, t_sampler, animated_uv);

    // Base water color (turquoise)
    let water_color = tex_color.rgb * in.color;

    // Reduced fresnel effect (water's reflectance, scaled down)
    let fresnel_factor = fresnel(in.view_direction, in.normal, 0.02) * 0.3;

    // Mix water color with subtle reflection
    let reflection_color = vec3<f32>(0.7, 0.8, 0.9); // Subtle sky-like reflection
    let final_color = mix(water_color, reflection_color, fresnel_factor);

    // Apply lighting
    let lit_color = final_color * lighting;

    // More consistent transparency
    let alpha = 0.5 + fresnel_factor * 0.15;

    return vec4<f32>(lit_color, alpha);
}
