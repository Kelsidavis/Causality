// PBR-style vertex and fragment shader

struct Uniforms {
    view_proj: mat4x4<f32>,
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
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    // Transform vertex position using push constant model matrix
    let world_position = push.model * vec4<f32>(in.position, 1.0);
    out.clip_position = uniforms.view_proj * world_position;
    out.world_position = world_position.xyz;

    // Transform normal (should use normal matrix, but for now just rotate)
    let normal_matrix = mat3x3<f32>(
        push.model[0].xyz,
        push.model[1].xyz,
        push.model[2].xyz,
    );
    out.normal = normalize(normal_matrix * in.normal);

    out.tex_coord = in.tex_coord;
    out.color = in.color;

    // Calculate shadow position
    out.shadow_position = shadow_uniforms.light_space_matrix * world_position;

    return out;
}

// Simple shadow calculation without PCF
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

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Simple lighting calculation
    // Light direction matches shadow map: lower angle for more visible shadows
    let light_dir = normalize(vec3<f32>(-0.8, 0.5, -0.4));
    let ambient = 0.3;
    let diffuse = max(dot(in.normal, light_dir), 0.0);

    // Calculate shadow
    let shadow = calculate_shadow(in.shadow_position);

    // Apply shadow to diffuse light (ambient not affected by shadow)
    let lighting = ambient + diffuse * 0.7 * shadow;

    // Sample texture
    let tex_color = textureSample(t_texture, t_sampler, in.tex_coord);

    // Combine texture with vertex color and lighting
    let base_color = tex_color.rgb * in.color;

    return vec4<f32>(base_color * lighting, tex_color.a);
}
