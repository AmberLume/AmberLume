import "bindings.slang";
import "sh.slang";
import "sky.slang";

float3 ibl_diffuse(uint sh_descriptor_id, float3 n) {
    return max(sh_texture_irradiance(sh_descriptor_id, n), float3(0.0));
}

float3 ibl_specular(uint brdf_lut_descriptor_id, uint sh_descriptor_id, float3 n, float3 v, float roughness, float3 f0, float3 sun_direction, float time) {
    float3 r = reflect(-v, n);
    float n_dot_v = max(dot(n, v), 0.0);

    float3 sky = procedural_sky(r, sun_direction, time);
    float3 blurred = ibl_diffuse(sh_descriptor_id, r);
    float3 prefiltered = lerp(sky, blurred, roughness * roughness);

    float2 brdf = graph_textures[brdf_lut_descriptor_id].Sample(samplers[SAMPLER_LINEAR_CLAMP], float2(n_dot_v, roughness)).rg;

    return prefiltered * (f0 * brdf.x + brdf.y);
}
