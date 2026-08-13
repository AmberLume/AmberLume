#version 460
#extension GL_EXT_samplerless_texture_functions : enable

#include "../bindings.glsl"
#include "../common.glsl"
#include "../sh.glsl"
#include "push_constants.glsl"

layout(location = 0) in vec2 in_uv;

layout(location = 0) out vec4 out_color;

vec3 view_direction(mat4 inverse_view_projection, vec2 ndc) {
    vec4 near_point = inverse_view_projection * vec4(ndc, 1.0, 1.0);
    vec4 far_point = inverse_view_projection * vec4(ndc, 0.0, 1.0);

    return normalize(far_point.xyz / far_point.w - near_point.xyz / near_point.w);
}

vec3 sample_sh_irradiance(uint id, vec3 dir) {
    vec3 c[9];
    for (int i = 0; i < 9; i++) {
        c[i] = texelFetch(graph_textures[id], ivec2(i, 0), 0).rgb;
    }

    return sh_irradiance(c, dir);
}

void main() {
    Scene scene = SceneBuffer(push_constants.scene_buffer_device_address).data;

    uint id = push_constants.texture_index;
    uint kind = push_constants.layer_kind;

    vec3 color;
    if (kind == DEBUG_LAYER_SH_IRRADIANCE) {
        color = sample_sh_irradiance(id, view_direction(scene.main_camera.inverse_view_projection, in_uv * 2.0 - 1.0));
    } else {
        vec4 sampled = texture(sampler2D(graph_textures[id], samplers[SAMPLER_LINEAR_CLAMP]), in_uv);

        if (kind == DEBUG_LAYER_VELOCITY) {
            color = vec3(0.5 + sampled.rg * 20.0, 0.5);
        } else if (kind == DEBUG_LAYER_NORMAL) {
            color = sampled.rgb * 0.5 + 0.5;
        } else if (kind == DEBUG_LAYER_HIZ_NEAR) {
            color = vec3(sampled.r);
        } else if (kind == DEBUG_LAYER_HIZ_FAR) {
            color = vec3(sampled.g);
        } else if (kind == DEBUG_LAYER_SHADOW) {
            color = push_constants.shadow_colored == 1u ? sampled.rgb : vec3(sampled.r);
        } else {
            color = vec3(sampled.r);
        }
    }

    out_color = vec4(color, 1.0);
}
