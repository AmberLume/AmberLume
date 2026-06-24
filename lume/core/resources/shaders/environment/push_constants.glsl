#ifndef PUSH_CONSTANTS_GLSL
#define PUSH_CONSTANTS_GLSL

layout(push_constant, std430) uniform PushConstants {
    mat4 inverse_view_projection;

    vec3 sun_direction;
    float time;
} push_constants;

#endif
