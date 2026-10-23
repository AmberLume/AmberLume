import "math.slang";
import "ibl.slang";

static const float PBR_MIN_ROUGHNESS = 0.045;

struct PbrResponse {
    float3 diffuse;
    float3 specular;
};

float3 pbr_f0(float3 albedo, float metallic) {
    return lerp(float3(0.04), albedo, metallic);
}

float3 fresnel_schlick(float cos_theta, float3 f0, float3 f90) {
    return f0 + (f90 - f0) * pow(clamp(1.0 - cos_theta, 0.0, 1.0), 5.0);
}

float3 fresnel_schlick_roughness(float cos_theta, float3 f0, float roughness) {
    return fresnel_schlick(cos_theta, f0, max(float3(1.0 - roughness), f0));
}

float specular_occlusion(float n_dot_v, float ambient_occlusion, float roughness) {
    return clamp(pow(n_dot_v + ambient_occlusion, exp2(-16.0 * roughness - 1.0)) - 1.0 + ambient_occlusion, 0.0, 1.0);
}

PbrResponse pbr_direct(float3 n, float3 v, float3 l, float3 albedo, float roughness, float metallic) {
    float3 h = normalize(v + l);

    float3 f0 = pbr_f0(albedo, metallic);

    float a2 = pow(roughness, 4.0);
    float n_dot_h = max(dot(n, h), 0.0);
    float n_dot_v = max(dot(n, v), 0.0);
    float n_dot_l = max(dot(n, l), 0.0);

    float denom = n_dot_h * n_dot_h * (a2 - 1.0) + 1.0;
    float ndf = a2 / (PI * denom * denom);

    float k = pow(roughness + 1.0, 2.0) / 8.0;
    float g = (n_dot_v / (n_dot_v * (1.0 - k) + k)) * (n_dot_l / (n_dot_l * (1.0 - k) + k));

    float3 f = fresnel_schlick(max(dot(h, v), 0.0), f0, float3(1.0));

    float3 k_d = (float3(1.0) - f) * (1.0 - metallic);

    PbrResponse response;
    response.diffuse = k_d * albedo / PI;
    response.specular = (ndf * g * f) / (4.0 * n_dot_v * n_dot_l + 0.0001);

    return response;
}

float pbr_reflectance(float3 n, float3 v, float3 albedo, float roughness, float metallic) {
    float3 f = fresnel_schlick_roughness(max(dot(n, v), 0.0), pbr_f0(albedo, metallic), roughness);

    return max(max(f.r, f.g), f.b);
}

PbrResponse pbr_ambient(
    uint brdf_lut_descriptor_id,
    uint sh_descriptor_id,
    float3 n,
    float3 v,
    float3 albedo,
    float roughness,
    float metallic,
    float3 sun_direction,
    float time
) {
    float3 f0 = pbr_f0(albedo, metallic);
    float n_dot_v = max(dot(n, v), 0.0);

    float3 f_ibl = fresnel_schlick_roughness(n_dot_v, f0, roughness);
    float3 k_d_ibl = (float3(1.0) - f_ibl) * (1.0 - metallic);

    PbrResponse response;
    response.diffuse = k_d_ibl * ibl_diffuse(sh_descriptor_id, n) * albedo;
    response.specular = ibl_specular(brdf_lut_descriptor_id, sh_descriptor_id, n, v, roughness, f0, sun_direction, time);

    return response;
}
