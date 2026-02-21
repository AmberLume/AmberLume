#ifndef PUSH_CONSANTS_GLSL
#define PUSH_CONSANTS_GLSL

#include "../common.glsl"

layout(push_constant) uniform PushConstants {
    vec4 frustum_planes[6];

    uint64_t indirect_buffer_device_address;

    uint64_t entity_buffer_device_address;

    uint64_t draw_data_buffer_device_address;
    uint64_t draw_count_buffer_device_address;
    
    uint64_t submesh_buffer_device_address;
    uint64_t model_buffer_device_address;

    uint64_t gpu_render_stats_buffer_device_address;

    uint entity_count;
} push_constants;

#endif