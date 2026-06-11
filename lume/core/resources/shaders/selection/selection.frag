#version 460
#extension GL_EXT_nonuniform_qualifier : enable
#extension GL_EXT_samplerless_texture_functions : enable

#include "push_constants.glsl"

layout(set = 0, binding = 0) uniform utexture2D entity_ids[];

layout(location = 0) out vec4 out_color;

void main() {
    ivec2 coord = ivec2(gl_FragCoord.xy);
    uint id = texelFetch(entity_ids[nonuniformEXT(push_constants.entity_id_texture)], coord, 0).r;

    float selected = (id == push_constants.selected_entity) ? 1.0 : 0.0;

    float diagonal = gl_FragCoord.x + gl_FragCoord.y;
    float wave = 0.5 + 0.5 * cos(diagonal / push_constants.stripe_width * 6.2831853);
    float stripe = pow(wave, 3.0);

    out_color = vec4(push_constants.color.rgb, push_constants.color.a * stripe * selected);
}
