// Composite shader - combines scene and bloom

// Scene texture (bind group 0)
@group(0) @binding(0)
var scene_texture: texture_2d<f32>;

@group(0) @binding(1)
var scene_sampler: sampler;

// Bloom texture (bind group 1)
@group(1) @binding(0)
var bloom_texture: texture_2d<f32>;

@group(1) @binding(1)
var bloom_sampler: sampler;

// Settings
struct Settings {
    bloom_intensity: f32,
    bloom_enabled: f32, // 0.0 or 1.0
    _padding: vec2<f32>,
}

var<push_constant> settings: Settings;

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
    // Sample scene color
    let scene_color = textureSample(scene_texture, scene_sampler, in.tex_coords);

    // Sample bloom color
    let bloom_color = textureSample(bloom_texture, bloom_sampler, in.tex_coords);

    // Additive blending: scene + bloom * intensity
    // Only add bloom if enabled
    let bloom_contribution = bloom_color.rgb * settings.bloom_intensity * settings.bloom_enabled;
    let final_color = scene_color.rgb + bloom_contribution;

    return vec4<f32>(final_color, scene_color.a);
}
