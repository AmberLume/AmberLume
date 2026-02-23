#ifndef PUSH_CONSANTS_GLSL
#define PUSH_CONSANTS_GLSL

#include "../common.glsl"

layout(push_constant, std430) uniform PushConstants {
    mat4 light_space_matrix;

    uint64_t entity_buffer_device_address;
    uint64_t vertex_buffer_device_address;
} push_constants;

#endif