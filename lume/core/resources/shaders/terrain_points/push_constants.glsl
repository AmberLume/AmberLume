#ifndef TERRAIN_POINTS_PUSH_CONSTANTS_GLSL
#define TERRAIN_POINTS_PUSH_CONSTANTS_GLSL

#include "../common.glsl"

layout(push_constant) uniform PushConstants {
    uint64_t scene_buffer_device_address;
    uint64_t chunk_buffer_device_address;
    uint64_t mesh_vertex_buffer_device_address;
    uint64_t mesh_buffer_device_address;
    uint64_t submesh_buffer_device_address;

    uint node_count;
    float point_size;
    float viewport_height;
} push_constants;

#endif
