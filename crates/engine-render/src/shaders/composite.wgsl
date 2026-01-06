// Composite shader - combines scene and bloom

@group(0) @binding(0)
var scene_texture: texture_2d<f32>;

@group(0) @binding(1)
var scene_sampler: sampler;

// Note: For bloom compositing, we'd need a second bind group for the bloom texture
// This is a simplified version that just passes through the scene

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

// Fullscreen triangle vertices
const VERTICES = array<vec2<f32>, 3>(
    vec2<f32>(-1.0, -1.0),
    vec2<f32>(3.0, -1.0),
    vec2<f32>(-1.0, 3.0),
);

const TEX_COORDS = array<vec2<f32>, 3>(
    vec2<f32>(0.0, 1.0),
    vec2<f32>(2.0, 1.0),
    vec2<f32>(0.0, -1.0),
);

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(VERTICES[vertex_index], 0.0, 1.0);
    out.tex_coords = TEX_COORDS[vertex_index];
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let scene_color = textureSample(scene_texture, scene_sampler, in.tex_coords);

    // In a full implementation, we'd sample the bloom texture here and add it
    // let bloom_color = textureSample(bloom_texture, bloom_sampler, in.tex_coords);
    // let bloom_intensity = 0.3;
    // let final_color = scene_color + bloom_color * bloom_intensity;

    // For now, just pass through the scene
    return scene_color;
}
