// Water shader with transparency, waves, and fresnel

struct Uniforms {
    view_proj: mat4x4<f32>,
    camera_pos: vec3<f32>,
    time: f32,
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

// Wave parameters
const WAVE_SPEED: f32 = 0.5;
const WAVE_FREQUENCY: f32 = 2.0;
const WAVE_AMPLITUDE: f32 = 0.1;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    // Apply wave animation to vertex position
    var animated_pos = in.position;
    let wave_phase = (in.position.x + in.position.z) * WAVE_FREQUENCY + uniforms.time * WAVE_SPEED;
    animated_pos.y += sin(wave_phase) * WAVE_AMPLITUDE;

    // Transform vertex position using push constant model matrix
    let world_position = push.model * vec4<f32>(animated_pos, 1.0);
    out.clip_position = uniforms.view_proj * world_position;
    out.world_position = world_position.xyz;

    // Calculate wave normal (animated)
    let wave_normal_offset = cos(wave_phase) * WAVE_AMPLITUDE * WAVE_FREQUENCY;
    var wave_normal = vec3<f32>(-wave_normal_offset, 1.0, -wave_normal_offset);
    wave_normal = normalize(wave_normal);

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

// Calculate shadow with PCF
fn calculate_shadow(shadow_pos: vec4<f32>) -> f32 {
    var proj_coords = shadow_pos.xyz / shadow_pos.w;
    proj_coords = proj_coords * 0.5 + 0.5;

    if proj_coords.x < 0.0 || proj_coords.x > 1.0 ||
       proj_coords.y < 0.0 || proj_coords.y > 1.0 ||
       proj_coords.z > 1.0 {
        return 1.0;
    }

    var shadow = 0.0;
    let texel_size = 1.0 / 2048.0;

    for (var x = -1; x <= 1; x++) {
        for (var y = -1; y <= 1; y++) {
            let offset = vec2<f32>(f32(x), f32(y)) * texel_size;
            shadow += textureSampleCompare(
                shadow_texture,
                shadow_sampler,
                proj_coords.xy + offset,
                proj_coords.z
            );
        }
    }

    return shadow / 9.0;
}

// Fresnel effect (Schlick approximation)
fn fresnel(view_dir: vec3<f32>, normal: vec3<f32>, f0: f32) -> f32 {
    let cos_theta = max(dot(view_dir, normal), 0.0);
    return f0 + (1.0 - f0) * pow(1.0 - cos_theta, 5.0);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Lighting
    let light_dir = normalize(vec3<f32>(0.5, 1.0, 0.3));
    let ambient = 0.2;
    let diffuse = max(dot(in.normal, light_dir), 0.0);

    // Calculate shadow
    let shadow = calculate_shadow(in.shadow_position);

    // Apply shadow to diffuse light
    let lighting = ambient + diffuse * 0.8 * shadow;

    // Sample texture with animated UVs
    let animated_uv = in.tex_coord + vec2<f32>(
        uniforms.time * 0.02,
        uniforms.time * 0.01
    );
    let tex_color = textureSample(t_texture, t_sampler, animated_uv);

    // Base water color (turquoise)
    let water_color = tex_color.rgb * in.color;

    // Fresnel effect (0.02 = water's reflectance at normal incidence)
    let fresnel_factor = fresnel(in.view_direction, in.normal, 0.02);

    // Mix water color with reflection color based on fresnel
    let reflection_color = vec3<f32>(0.8, 0.9, 1.0); // Sky-like reflection
    let final_color = mix(water_color, reflection_color, fresnel_factor);

    // Apply lighting
    let lit_color = final_color * lighting;

    // Alpha based on fresnel (more transparent when viewing at grazing angles)
    let alpha = 0.6 + fresnel_factor * 0.3;

    return vec4<f32>(lit_color, alpha);
}
