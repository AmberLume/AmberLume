import "common.slang";
import "bindings.slang";
import "pbr.slang";

struct MaterialSurface {
    float3 normal;
    float4 albedo;
    float ambient_occlusion;
    float roughness;
    float metallic;
};

MaterialSurface sample_material_surface(Material material, float2 uv, float3x3 tbn, float mip_bias) {
    SamplerState linear_clamp = samplers[SAMPLER_LINEAR_CLAMP];

    float3 normal_sample = textures[NonUniformResourceIndex(material.normal_texture_index)].SampleBias(linear_clamp, uv, mip_bias).rgb;
    float4 occlusion_roughness_metallic = textures[NonUniformResourceIndex(material.occlusion_roughness_metallic_texture_index)].SampleBias(linear_clamp, uv, mip_bias);
    float4 color = textures[NonUniformResourceIndex(material.color_texture_index)].SampleBias(linear_clamp, uv, mip_bias);

    MaterialSurface surface;
    surface.normal = normalize(mul(normal_sample * 2.0 - 1.0, tbn));
    surface.albedo = color * material.base_color_factor;
    surface.ambient_occlusion = occlusion_roughness_metallic.r;
    surface.roughness = max(occlusion_roughness_metallic.g * material.roughness_factor, PBR_MIN_ROUGHNESS);
    surface.metallic = occlusion_roughness_metallic.b * material.metallic_factor;

    return surface;
}
