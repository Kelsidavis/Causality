// Skybox shader - renders cubemap at far plane

struct CameraUniforms {
    view_proj: mat4x4<f32>,
    view_proj_inverse: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniforms;

@group(1) @binding(0)
var skybox_texture: texture_cube<f32>;

@group(1) @binding(1)
var skybox_sampler: sampler;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec3<f32>,
}

// Fullscreen triangle vertices
const VERTICES = array<vec2<f32>, 3>(
    vec2<f32>(-1.0, -1.0),
    vec2<f32>(3.0, -1.0),
    vec2<f32>(-1.0, 3.0),
);

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;

    let pos = VERTICES[vertex_index];
    out.clip_position = vec4<f32>(pos, 1.0, 1.0);

    // Convert clip space to view direction using pre-computed inverse
    // The inverse is computed on the CPU side to avoid WGSL limitations
    let world_pos = camera.view_proj_inverse * vec4<f32>(pos, 1.0, 1.0);

    out.tex_coords = world_pos.xyz;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(skybox_texture, skybox_sampler, normalize(in.tex_coords));
}
