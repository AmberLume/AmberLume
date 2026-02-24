#ifndef PUSH_CONSANTS_GLSL
#define PUSH_CONSANTS_GLSL

#include "../common.glsl"

layout(push_constant, std430) uniform PushConstants {
    mat4 screen_to_light;

    uint depth_descriptor_id;
    uint global_shadow_descriptor_id;
} push_constants;

#endif