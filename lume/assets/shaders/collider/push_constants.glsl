#ifndef PUSH_CONSANTS_GLSL
#define PUSH_CONSANTS_GLSL

#include "../common.glsl"

layout(push_constant, std430) uniform PushConstants {
    mat4 projection_matrix;

    uint64_t collider_buffer_device_address;
} push_constants;

#endif