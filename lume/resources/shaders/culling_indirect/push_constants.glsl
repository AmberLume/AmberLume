#ifndef PUSH_CONSTANTS_GLSL
#define PUSH_CONSTANTS_GLSL

#include "../common.glsl"

layout(push_constant) uniform PushConstants {
    uint64_t culling_views_buffer_device_address;
    uint64_t entity_buffer_device_address;
    uint64_t mesh_buffer_device_address;
    uint64_t submesh_buffer_device_address;
    uint64_t meta_statistics_buffer_device_address;

    uint culling_views_count;
    uint entity_count;
} push_constants;

#endif