#ifndef PUSH_CONSTANTS_GLSL
#define PUSH_CONSTANTS_GLSL

#include "../common.glsl"

layout(push_constant, std430) uniform PushConstants {
    mat4 inverse_view_projection;

    float time;

    float frequency;
    float probability;
    float core_size;

    vec4 color_cool;
    vec4 color_warm;
} push_constants;

#endif
