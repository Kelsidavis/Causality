// Shadow mapping shader - depth-only pass

struct ShadowUniforms {
    light_space_matrix: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: ShadowUniforms;

// Push constants for per-object data (model matrix)
struct PushConstants {
    model: mat4x4<f32>,
}
var<push_constant> push: PushConstants;

struct VertexInput {
    @location(0) position: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let world_pos = push.model * vec4<f32>(in.position, 1.0);
    out.clip_position = uniforms.light_space_matrix * world_pos;
    return out;
}

// No fragment shader needed for depth-only pass
