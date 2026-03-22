#ifndef PUSH_CONSTANTS_GLSL
#define PUSH_CONSTANTS_GLSL

#include "../common.glsl"

layout(push_constant, std430) uniform PushConstants {
    mat4 view_projection;

    uint64_t physics_debug_vertices_buffer_device_address;
} push_constants;

#endif
