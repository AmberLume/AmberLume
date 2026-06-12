#version 460
#extension GL_EXT_samplerless_texture_functions : enable

#include "../bindings.glsl"
#include "push_constants.glsl"

layout(location = 0) out vec4 out_color;

void main() {
    ivec2 coord = ivec2(gl_FragCoord.xy);
    uint id = texelFetch(graph_utextures[nonuniformEXT(push_constants.entity_id_texture)], coord, 0).r;

    float selected = (id == push_constants.selected_entity) ? 1.0 : 0.0;

    float diagonal = gl_FragCoord.x + gl_FragCoord.y;
    float wave = 0.5 + 0.5 * cos(diagonal / push_constants.stripe_width * 6.2831853);
    float stripe = pow(wave, 3.0);

    out_color = vec4(push_constants.color.rgb, push_constants.color.a * stripe * selected);
}
