#version 460

#include "../common.glsl"
#include "push_constants.glsl"

layout(location = 0) in vec3 world_normal;
layout(location = 1) in vec4 current_clip;
layout(location = 2) in vec4 previous_clip;

layout(location = 0) out vec4 out_normal;
layout(location = 1) out vec2 out_velocity;

void main() {
    out_normal = vec4(normalize(world_normal), 0.0);

    vec2 current_ndc = current_clip.xy / current_clip.w;
    vec2 previous_ndc = previous_clip.xy / previous_clip.w;
    out_velocity = (previous_ndc - current_ndc) * 0.5;
}
