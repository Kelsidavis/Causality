// Tone mapping shader - ACES filmic tone mapping

@group(0) @binding(0)
var input_texture: texture_2d<f32>;

@group(0) @binding(1)
var input_sampler: sampler;

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

// ACES Filmic Tone Mapping
// https://github.com/TheRealMJP/BakingLab/blob/master/BakingLab/ACES.hlsl
fn aces_tonemap(color: vec3<f32>) -> vec3<f32> {
    let a = 2.51;
    let b = 0.03;
    let c = 2.43;
    let d = 0.59;
    let e = 0.14;
    return saturate((color * (a * color + b)) / (color * (c * color + d) + e));
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let hdr_color = textureSample(input_texture, input_sampler, in.tex_coords);

    // Apply exposure
    let exposed = hdr_color.rgb * 1.0; // Exposure could be a uniform

    // Apply ACES tone mapping
    let mapped = aces_tonemap(exposed);

    // Gamma correction
    let gamma_corrected = pow(mapped, vec3<f32>(1.0 / 2.2));

    return vec4<f32>(gamma_corrected, hdr_color.a);
}
