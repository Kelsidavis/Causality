// Bloom shader - bright pass extraction and gaussian blur

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

// Extract bright areas (threshold-based)
@fragment
fn fs_bright_pass(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(input_texture, input_sampler, in.tex_coords);

    // Calculate luminance
    let luminance = dot(color.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));

    // Threshold for bloom
    let threshold = 1.0;

    if luminance > threshold {
        return color;
    } else {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }
}

// Gaussian blur - horizontal pass
@fragment
fn fs_blur_h(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex_size = textureDimensions(input_texture);
    let texel_size = 1.0 / vec2<f32>(f32(tex_size.x), f32(tex_size.y));

    // 9-tap gaussian blur
    let weights = array<f32, 5>(
        0.227027, 0.1945946, 0.1216216, 0.054054, 0.016216
    );

    var result = textureSample(input_texture, input_sampler, in.tex_coords).rgb * weights[0];

    for (var i: i32 = 1; i < 5; i++) {
        let offset = vec2<f32>(f32(i) * texel_size.x, 0.0);
        result += textureSample(input_texture, input_sampler, in.tex_coords + offset).rgb * weights[i];
        result += textureSample(input_texture, input_sampler, in.tex_coords - offset).rgb * weights[i];
    }

    return vec4<f32>(result, 1.0);
}

// Gaussian blur - vertical pass
@fragment
fn fs_blur_v(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex_size = textureDimensions(input_texture);
    let texel_size = 1.0 / vec2<f32>(f32(tex_size.x), f32(tex_size.y));

    // 9-tap gaussian blur
    let weights = array<f32, 5>(
        0.227027, 0.1945946, 0.1216216, 0.054054, 0.016216
    );

    var result = textureSample(input_texture, input_sampler, in.tex_coords).rgb * weights[0];

    for (var i: i32 = 1; i < 5; i++) {
        let offset = vec2<f32>(0.0, f32(i) * texel_size.y);
        result += textureSample(input_texture, input_sampler, in.tex_coords + offset).rgb * weights[i];
        result += textureSample(input_texture, input_sampler, in.tex_coords - offset).rgb * weights[i];
    }

    return vec4<f32>(result, 1.0);
}
