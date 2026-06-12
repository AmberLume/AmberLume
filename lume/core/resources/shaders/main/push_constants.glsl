#ifndef PUSH_CONSTANTS_GLSL
#define PUSH_CONSTANTS_GLSL

#include "../common.glsl"

layout(push_constant, std430) uniform PushConstants {
    uint64_t scene_buffer_device_address;
    uint64_t draw_data_buffer_device_address;
    uint64_t vertex_buffer_device_address;
    uint64_t entity_buffer_device_address;
    uint64_t submesh_buffer_device_address;
    uint64_t material_buffer_device_address;
    uint64_t bone_transform_buffer_device_address;
    uint64_t shadow_cascades_buffer_device_address;

    uint shadow_array_descriptor_id;
    float shadow_bias;
    float shadow_normal_bias;
    float shadow_pcf_world_radius;
    uint shadow_pcf_sample_count;
    float shadow_cascade_blend_range;

    uint gtao_descriptor_id;
    uint gtao_enabled;
} push_constants;

#endif
