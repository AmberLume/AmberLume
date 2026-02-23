#ifndef PUSH_CONSANTS_GLSL
#define PUSH_CONSANTS_GLSL

#include "../common.glsl"

layout(push_constant) uniform PushConstants {
    uint64_t culling_views_buffer_device_address;

    uint64_t entity_buffer_device_address;

    uint64_t submesh_buffer_device_address;
    uint64_t model_buffer_device_address;

    uint64_t gpu_render_stats_buffer_device_address;

    uint culling_views_count;
    uint entity_count;
} push_constants;

#endif