#ifndef PBR_GLSL
#define PBR_GLSL

#include "ibl.glsl"

const float PBR_MIN_ROUGHNESS = 0.045;

struct PbrResponse {
    vec3 diffuse;
    vec3 specular;
};

vec3 pbr_f0(vec3 albedo, float metallic) {
    return mix(vec3(0.04), albedo, metallic);
}

PbrResponse pbr_direct(vec3 n, vec3 v, vec3 l, vec3 albedo, float roughness, float metallic) {
    vec3 h = normalize(v + l);

    vec3 f0 = pbr_f0(albedo, metallic);

    float a2 = pow(roughness, 4.0);
    float n_dot_h = max(dot(n, h), 0.0);
    float n_dot_v = max(dot(n, v), 0.0);
    float n_dot_l = max(dot(n, l), 0.0);

    float denom = (n_dot_h * n_dot_h * (a2 - 1.0) + 1.0);
    float ndf = a2 / (3.14159 * denom * denom);

    float k = pow(roughness + 1.0, 2.0) / 8.0;
    float g = (n_dot_v / (n_dot_v * (1.0 - k) + k)) * (n_dot_l / (n_dot_l * (1.0 - k) + k));

    vec3 f = f0 + (1.0 - f0) * pow(clamp(1.0 - max(dot(h, v), 0.0), 0.0, 1.0), 5.0);

    vec3 k_d = (vec3(1.0) - f) * (1.0 - metallic);

    PbrResponse response;
    response.diffuse = k_d * albedo / 3.14159;
    response.specular = (ndf * g * f) / (4.0 * n_dot_v * n_dot_l + 0.0001);

    return response;
}

PbrResponse pbr_ambient(
    uint brdf_lut_descriptor,
    uint sh_descriptor,
    vec3 n,
    vec3 v,
    vec3 albedo,
    float roughness,
    float metallic,
    vec3 sun_dir,
    float time
) {
    vec3 f0 = pbr_f0(albedo, metallic);
    float n_dot_v = max(dot(n, v), 0.0);

    vec3 f_ibl = fresnel_schlick_roughness(n_dot_v, f0, roughness);
    vec3 k_d_ibl = (vec3(1.0) - f_ibl) * (1.0 - metallic);

    PbrResponse response;
    response.diffuse = k_d_ibl * ibl_diffuse(sh_descriptor, n) * albedo;
    response.specular = ibl_specular(brdf_lut_descriptor, sh_descriptor, n, v, roughness, f0, sun_dir, time);

    return response;
}

#endif
