#ifndef IBL_GLSL
#define IBL_GLSL

#include "bindings.glsl"
#include "sh.glsl"
#include "sky.glsl"

vec3 ibl_diffuse(uint sh_descriptor, vec3 n) {
    vec3 c[9];
    for (int i = 0; i < 9; i++) {
        c[i] = texelFetch(graph_textures[sh_descriptor], ivec2(i, 0), 0).rgb;
    }

    return max(sh_irradiance(c, n), vec3(0.0));
}

vec3 fresnel_schlick_roughness(float cos_theta, vec3 f0, float roughness) {
    vec3 f90 = max(vec3(1.0 - roughness), f0);
    return f0 + (f90 - f0) * pow(clamp(1.0 - cos_theta, 0.0, 1.0), 5.0);
}

float specular_occlusion(float n_dot_v, float ao, float roughness) {
    return clamp(pow(n_dot_v + ao, exp2(-16.0 * roughness - 1.0)) - 1.0 + ao, 0.0, 1.0);
}

vec3 ibl_specular(uint brdf_lut_descriptor, uint sh_descriptor, vec3 n, vec3 v, float roughness, vec3 f0, vec3 sun_dir, float time) {
    vec3 r = reflect(-v, n);
    float n_dot_v = max(dot(n, v), 0.0);

    vec3 sky = procedural_sky(r, sun_dir, time, false);
    vec3 blurred = ibl_diffuse(sh_descriptor, r);
    vec3 prefiltered = mix(sky, blurred, roughness * roughness);

    vec2 brdf = texture(sampler2D(graph_textures[brdf_lut_descriptor], samplers[SAMPLER_LINEAR_CLAMP]), vec2(n_dot_v, roughness)).rg;

    return prefiltered * (f0 * brdf.x + brdf.y);
}

#endif
