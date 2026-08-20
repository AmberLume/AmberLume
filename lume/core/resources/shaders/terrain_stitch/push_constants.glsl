#ifndef TERRAIN_STITCH_PUSH_CONSTANTS_GLSL
#define TERRAIN_STITCH_PUSH_CONSTANTS_GLSL

#include "../common.glsl"

layout(push_constant) uniform PushConstants {
    uint64_t request_buffer_device_address;
    uint64_t edge_height_buffer_device_address;
    uint64_t vertex_buffer_device_address;
    uint64_t mesh_buffer_device_address;
    uint64_t submesh_buffer_device_address;

    uint node_count;
    uint nodes;
} push_constants;

#endif
