#ifndef PUSH_CONSTANTS_GLSL
#define PUSH_CONSTANTS_GLSL

#include "../common.glsl"

layout(push_constant) uniform PushConstants {
    uint64_t culling_views_buffer_device_address;
    uint64_t entity_buffer_device_address;
    uint64_t mesh_buffer_device_address;
    uint64_t submesh_buffer_device_address;
    uint64_t meta_statistics_buffer_device_address;

    uint64_t cull_requests_buffer_device_address;
    uint64_t material_buffer_device_address;
    uint64_t scene_buffer_device_address;

    uint view_count;
    uint entity_count;
    uint combine_views;
    uint request_count;
} push_constants;

#endif
