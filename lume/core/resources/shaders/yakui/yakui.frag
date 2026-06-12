#version 460

#include "../bindings.glsl"
#include "../common.glsl"
#include "push_constants.glsl"

layout(location = 0) in vec4 in_color;
layout(location = 1) in vec2 in_texcoord;

layout(location = 0) out vec4 out_color;

void main() {
    uint render_mode = push_constants.render_mode;

    if (render_mode == 0) {
        out_color = in_color;
    } else if (render_mode == 1) {
        vec4 texture = texture(sampler2D(textures[nonuniformEXT(push_constants.texture_index)], samplers[SAMPLER_LINEAR_CLAMP]), in_texcoord);

        out_color = in_color * texture.r;
    }
}
