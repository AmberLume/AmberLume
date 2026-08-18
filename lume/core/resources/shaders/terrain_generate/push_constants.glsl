#ifndef TERRAIN_GENERATE_PUSH_CONSTANTS_GLSL
#define TERRAIN_GENERATE_PUSH_CONSTANTS_GLSL

#include "../common.glsl"

layout(push_constant) uniform PushConstants {
    uint64_t request_buffer_device_address;
    uint64_t height_buffer_device_address;
    uint64_t vertex_buffer_device_address;

    uint node_count;
    uint nodes;
    uint window_stride;
    float cell_size;
} push_constants;

#endif
