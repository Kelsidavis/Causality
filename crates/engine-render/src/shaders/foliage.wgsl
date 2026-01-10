// Foliage shader - Instanced mesh rendering for vegetation
//
// Renders meshes with per-instance transforms and color tints.

struct CameraUniforms {
    view_proj: mat4x4<f32>,
    camera_pos: vec3<f32>,
    _padding: f32,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniforms;

struct VertexInput {
    // Mesh vertex data
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tex_coord: vec2<f32>,
    @location(3) color: vec3<f32>,
    // Instance data (model matrix as 4 columns)
    @location(4) model_col0: vec4<f32>,
    @location(5) model_col1: vec4<f32>,
    @location(6) model_col2: vec4<f32>,
    @location(7) model_col3: vec4<f32>,
    @location(8) color_tint: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) tex_coord: vec2<f32>,
    @location(3) color: vec3<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    // Reconstruct model matrix from columns
    let model = mat4x4<f32>(
        in.model_col0,
        in.model_col1,
        in.model_col2,
        in.model_col3
    );

    // Transform position
    let world_pos = model * vec4<f32>(in.position, 1.0);

    // Transform normal (using upper 3x3 of model matrix)
    let normal_matrix = mat3x3<f32>(
        in.model_col0.xyz,
        in.model_col1.xyz,
        in.model_col2.xyz
    );
    let world_normal = normalize(normal_matrix * in.normal);

    // Apply color tint to vertex color
    let tinted_color = in.color * in.color_tint.rgb;

    var out: VertexOutput;
    out.clip_position = camera.view_proj * world_pos;
    out.world_position = world_pos.xyz;
    out.world_normal = world_normal;
    out.tex_coord = in.tex_coord;
    out.color = tinted_color;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Simple directional lighting
    let light_dir = normalize(vec3<f32>(0.5, 1.0, 0.3));
    let ambient = 0.3;
    let diffuse = max(dot(in.world_normal, light_dir), 0.0) * 0.7;

    // Apply lighting to vertex color
    let lit_color = in.color * (ambient + diffuse);

    // Add slight variation based on height for more natural look
    let height_factor = clamp(in.world_position.y * 0.1, 0.0, 0.2);
    let final_color = lit_color + vec3<f32>(height_factor * 0.1, height_factor * 0.15, 0.0);

    return vec4<f32>(final_color, 1.0);
}
