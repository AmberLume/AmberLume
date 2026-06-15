#version 460
#extension GL_EXT_samplerless_texture_functions : enable

#include "../bindings.glsl"
#include "push_constants.glsl"

layout(location = 0) in vec2 in_uv;

layout(location = 0) out vec4 out_color;

void main() {
    uint id = nonuniformEXT(push_constants.texture_index);
    vec4 sampled = texture(sampler2D(graph_textures[id], samplers[SAMPLER_LINEAR_CLAMP]), in_uv);

    vec3 color;
    if (push_constants.layer_kind == 1u) {
        color = vec3(0.5 + sampled.rg * 20.0, 0.5);
    } else if (push_constants.layer_kind == 2u) {
        color = sampled.rgb * 0.5 + 0.5;
    } else {
        color = vec3(sampled.r);
    }

    out_color = vec4(color, 1.0);
}
