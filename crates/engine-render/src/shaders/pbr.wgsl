// PBR-style vertex and fragment shader

struct Uniforms {
    view_proj: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

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

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Simple lighting calculation
    let light_dir = normalize(vec3<f32>(0.5, 1.0, 0.3));
    let ambient = 0.3;
    let diffuse = max(dot(in.normal, light_dir), 0.0);
    let lighting = ambient + diffuse * 0.7;

    // Use vertex color or default white
    let base_color = in.color;

    return vec4<f32>(base_color * lighting, 1.0);
}
