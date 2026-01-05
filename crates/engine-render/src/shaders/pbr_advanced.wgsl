// Advanced PBR shader with metallic-roughness workflow
// Based on physically-based rendering principles

struct Uniforms {
    view_proj: mat4x4<f32>,
    model: mat4x4<f32>,
    camera_pos: vec3<f32>,
    _padding: f32,
}

struct Light {
    position: vec3<f32>,
    _padding1: f32,
    color: vec3<f32>,
    intensity: f32,
}

struct MaterialParams {
    base_color: vec3<f32>,
    metallic: f32,
    roughness: f32,
    _padding: vec3<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(0) @binding(1)
var<uniform> material: MaterialParams;

@group(0) @binding(2)
var<uniform> lights: array<Light, 4>;

const PI: f32 = 3.14159265359;

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

    let world_position = uniforms.model * vec4<f32>(in.position, 1.0);
    out.clip_position = uniforms.view_proj * world_position;
    out.world_position = world_position.xyz;

    // Proper normal transformation (should be inverse transpose of model matrix)
    let normal_matrix = mat3x3<f32>(
        uniforms.model[0].xyz,
        uniforms.model[1].xyz,
        uniforms.model[2].xyz,
    );
    out.normal = normalize(normal_matrix * in.normal);

    out.tex_coord = in.tex_coord;
    out.color = in.color;

    return out;
}

// Trowbridge-Reitz GGX normal distribution function
fn distribution_ggx(N: vec3<f32>, H: vec3<f32>, roughness: f32) -> f32 {
    let a = roughness * roughness;
    let a2 = a * a;
    let NdotH = max(dot(N, H), 0.0);
    let NdotH2 = NdotH * NdotH;

    let nom = a2;
    var denom = (NdotH2 * (a2 - 1.0) + 1.0);
    denom = PI * denom * denom;

    return nom / max(denom, 0.0001);
}

// Schlick-GGX geometry function
fn geometry_schlick_ggx(NdotV: f32, roughness: f32) -> f32 {
    let r = roughness + 1.0;
    let k = (r * r) / 8.0;

    let nom = NdotV;
    let denom = NdotV * (1.0 - k) + k;

    return nom / max(denom, 0.0001);
}

// Smith's method for geometry obstruction
fn geometry_smith(N: vec3<f32>, V: vec3<f32>, L: vec3<f32>, roughness: f32) -> f32 {
    let NdotV = max(dot(N, V), 0.0);
    let NdotL = max(dot(N, L), 0.0);
    let ggx2 = geometry_schlick_ggx(NdotV, roughness);
    let ggx1 = geometry_schlick_ggx(NdotL, roughness);

    return ggx1 * ggx2;
}

// Fresnel-Schlick approximation
fn fresnel_schlick(cos_theta: f32, F0: vec3<f32>) -> vec3<f32> {
    return F0 + (vec3<f32>(1.0) - F0) * pow(max(1.0 - cos_theta, 0.0), 5.0);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let N = normalize(in.normal);
    let V = normalize(uniforms.camera_pos - in.world_position);

    // Base color from material and vertex color
    let albedo = material.base_color * in.color;

    // Calculate reflectance at normal incidence
    // For dielectrics F0 is usually 0.04, for metals it's the albedo color
    var F0 = vec3<f32>(0.04);
    F0 = mix(F0, albedo, material.metallic);

    // Reflectance equation
    var Lo = vec3<f32>(0.0);

    // Calculate per-light radiance
    for (var i = 0u; i < 4u; i = i + 1u) {
        let light = lights[i];

        // Skip if light has no intensity
        if (light.intensity <= 0.0) {
            continue;
        }

        let L = normalize(light.position - in.world_position);
        let H = normalize(V + L);

        let distance = length(light.position - in.world_position);
        let attenuation = 1.0 / (distance * distance);
        let radiance = light.color * light.intensity * attenuation;

        // Cook-Torrance BRDF
        let NDF = distribution_ggx(N, H, material.roughness);
        let G = geometry_smith(N, V, L, material.roughness);
        let F = fresnel_schlick(max(dot(H, V), 0.0), F0);

        let numerator = NDF * G * F;
        let NdotL = max(dot(N, L), 0.0);
        let NdotV = max(dot(N, V), 0.0);
        let denominator = 4.0 * NdotV * NdotL + 0.0001;
        let specular = numerator / denominator;

        // Energy conservation
        let kS = F;
        var kD = vec3<f32>(1.0) - kS;
        kD = kD * (1.0 - material.metallic);

        // Add to outgoing radiance
        Lo = Lo + (kD * albedo / PI + specular) * radiance * NdotL;
    }

    // Ambient lighting (simplified IBL)
    let ambient = vec3<f32>(0.03) * albedo;
    let color = ambient + Lo;

    // Tone mapping (Reinhard)
    let mapped = color / (color + vec3<f32>(1.0));

    // Gamma correction
    let gamma_corrected = pow(mapped, vec3<f32>(1.0 / 2.2));

    return vec4<f32>(gamma_corrected, 1.0);
}
