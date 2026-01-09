// Advanced PBR shader with Cook-Torrance BRDF and normal mapping
// Supports metallic-roughness workflow with multiple textures

// Global uniforms (view-projection and camera position)
struct Uniforms {
    view_proj: mat4x4<f32>,
    camera_pos: vec3<f32>,
    _padding: f32,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

// Material uniforms and textures
struct MaterialUniforms {
    base_color: vec4<f32>,
    emissive_color: vec3<f32>,
    metallic: f32,
    roughness: f32,
    ao_factor: f32,
    emissive_strength: f32,
    alpha_cutoff: f32,
    _padding: vec3<f32>,
}

@group(1) @binding(0)
var<uniform> material: MaterialUniforms;

// Albedo texture
@group(1) @binding(1)
var albedo_texture: texture_2d<f32>;
@group(1) @binding(2)
var albedo_sampler: sampler;

// Normal map
@group(1) @binding(3)
var normal_texture: texture_2d<f32>;
@group(1) @binding(4)
var normal_sampler: sampler;

// Metallic-roughness texture (R=AO, G=Roughness, B=Metallic in some formats)
@group(1) @binding(5)
var metallic_roughness_texture: texture_2d<f32>;
@group(1) @binding(6)
var metallic_roughness_sampler: sampler;

// Ambient occlusion texture
@group(1) @binding(7)
var ao_texture: texture_2d<f32>;
@group(1) @binding(8)
var ao_sampler: sampler;

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

// Push constants for per-object model matrix
struct PushConstants {
    model: mat4x4<f32>,
}
var<push_constant> push: PushConstants;

const PI: f32 = 3.14159265359;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tex_coord: vec2<f32>,
    @location(3) color: vec3<f32>,
    @location(4) tangent: vec4<f32>,      // w = handedness
    @location(5) bitangent: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tex_coord: vec2<f32>,
    @location(3) color: vec3<f32>,
    @location(4) tangent: vec3<f32>,
    @location(5) bitangent: vec3<f32>,
    @location(6) shadow_position: vec4<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    // Transform to world space using push constant model matrix
    let world_position = push.model * vec4<f32>(in.position, 1.0);
    out.clip_position = uniforms.view_proj * world_position;
    out.world_position = world_position.xyz;

    // Transform TBN basis to world space
    let normal_matrix = mat3x3<f32>(
        push.model[0].xyz,
        push.model[1].xyz,
        push.model[2].xyz,
    );

    out.normal = normalize(normal_matrix * in.normal);
    out.tangent = normalize(normal_matrix * in.tangent.xyz);
    out.bitangent = normalize(normal_matrix * in.bitangent);

    out.tex_coord = in.tex_coord;
    out.color = in.color;

    // Calculate shadow position
    out.shadow_position = shadow_uniforms.light_space_matrix * world_position;

    return out;
}

// Sample normal map and transform to world space
fn sample_normal_map(uv: vec2<f32>, T: vec3<f32>, B: vec3<f32>, N: vec3<f32>) -> vec3<f32> {
    // Sample normal map
    let normal_sample = textureSample(normal_texture, normal_sampler, uv).xyz;

    // Convert from [0,1] to [-1,1] range
    let tangent_normal = normal_sample * 2.0 - 1.0;

    // Construct TBN matrix
    let TBN = mat3x3<f32>(
        normalize(T),
        normalize(B),
        normalize(N)
    );

    // Transform normal from tangent space to world space
    return normalize(TBN * tangent_normal);
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

// Calculate shadow with PCF
fn calculate_shadow(shadow_pos: vec4<f32>) -> f32 {
    // Perform perspective divide
    var proj_coords = shadow_pos.xyz / shadow_pos.w;

    // Transform to [0, 1] range
    proj_coords = proj_coords * 0.5 + 0.5;

    // Check if outside shadow map bounds
    if proj_coords.x < 0.0 || proj_coords.x > 1.0 ||
       proj_coords.y < 0.0 || proj_coords.y > 1.0 ||
       proj_coords.z > 1.0 {
        return 1.0; // Not in shadow
    }

    // PCF (Percentage Closer Filtering) for soft shadows
    var shadow = 0.0;
    let texel_size = 1.0 / 2048.0; // SHADOW_MAP_SIZE

    for (var x = -1; x <= 1; x++) {
        for (var y = -1; y <= 1; y++) {
            let offset = vec2<f32>(f32(x), f32(y)) * texel_size;
            shadow += textureSampleCompare(
                shadow_texture,
                shadow_sampler,
                proj_coords.xy + offset,
                proj_coords.z
            );
        }
    }

    return shadow / 9.0; // Average of 9 samples
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Sample normal map and construct world-space normal
    let N = sample_normal_map(in.tex_coord, in.tangent, in.bitangent, in.normal);
    let V = normalize(uniforms.camera_pos - in.world_position);

    // Sample textures
    let albedo_sample = textureSample(albedo_texture, albedo_sampler, in.tex_coord);
    let albedo = albedo_sample.rgb * material.base_color.rgb * in.color;

    // Sample metallic and roughness (using material uniforms as fallback)
    let mr_sample = textureSample(metallic_roughness_texture, metallic_roughness_sampler, in.tex_coord);
    let metallic = material.metallic; // Could use mr_sample.b for texture-based metallic
    let roughness = material.roughness; // Could use mr_sample.g for texture-based roughness

    // Sample ambient occlusion
    let ao_sample = textureSample(ao_texture, ao_sampler, in.tex_coord).r;
    let ao = ao_sample * material.ao_factor;

    // Calculate reflectance at normal incidence
    // For dielectrics F0 is ~0.04, for metals it's the albedo color
    var F0 = vec3<f32>(0.04);
    F0 = mix(F0, albedo, metallic);

    // Directional light (simple for now - could be made configurable)
    let light_dir = normalize(vec3<f32>(0.5, 1.0, 0.3));
    let light_color = vec3<f32>(1.0, 1.0, 1.0);
    let light_intensity = 3.0;

    let L = light_dir;
    let H = normalize(V + L);

    // Cook-Torrance BRDF
    let NDF = distribution_ggx(N, H, roughness);
    let G = geometry_smith(N, V, L, roughness);
    let F = fresnel_schlick(max(dot(H, V), 0.0), F0);

    let numerator = NDF * G * F;
    let NdotL = max(dot(N, L), 0.0);
    let NdotV = max(dot(N, V), 0.0);
    let denominator = 4.0 * NdotV * NdotL + 0.0001;
    let specular = numerator / denominator;

    // Energy conservation
    let kS = F; // Specular reflection
    var kD = vec3<f32>(1.0) - kS; // Diffuse reflection
    kD = kD * (1.0 - metallic); // Metals have no diffuse

    // Calculate radiance
    let radiance = light_color * light_intensity;

    // Apply shadow
    let shadow = calculate_shadow(in.shadow_position);

    // Outgoing light (diffuse + specular)
    var Lo = (kD * albedo / PI + specular) * radiance * NdotL * shadow;

    // Ambient lighting (simplified IBL)
    let ambient = vec3<f32>(0.03) * albedo * ao;

    // Add emissive
    let emissive = material.emissive_color * material.emissive_strength;

    var color = ambient + Lo + emissive;

    // Simple tone mapping (Reinhard)
    color = color / (color + vec3<f32>(1.0));

    // Gamma correction
    color = pow(color, vec3<f32>(1.0 / 2.2));

    return vec4<f32>(color, albedo_sample.a * material.base_color.a);
}
