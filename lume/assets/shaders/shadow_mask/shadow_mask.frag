#version 460

#extension GL_EXT_nonuniform_qualifier : enable

#include "../common.glsl"
#include "push_constants.glsl"

layout(location = 0) in vec2 in_uv;
layout(location = 0) out float out_mask;

layout(set = 0, binding = 0) uniform sampler2D global_textures[];
layout(set = 0, binding = 1) uniform sampler2DShadow global_shadow_textures[];

void main() {
    float depth = texture(global_textures[nonuniformEXT(push_constants.depth_descriptor_id)], in_uv).r;

    if (depth >= 1.0) {
        out_mask = 1.0;
        return;
    }

    vec4 screen_position = vec4(in_uv * 2.0 - 1.0, depth, 1.0);
    vec4 shadow_position = push_constants.screen_to_light * screen_position;
    vec3 projection = shadow_position.xyz / shadow_position.w;

    vec3 shadow_coordinates = vec3(projection.xy * 0.5 + 0.5, projection.z);

    out_mask = texture(global_shadow_textures[nonuniformEXT(push_constants.global_shadow_descriptor_id)], shadow_coordinates);
}
