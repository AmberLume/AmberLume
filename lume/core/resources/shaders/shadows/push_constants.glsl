#ifndef PUSH_CONSTANTS_GLSL
#define PUSH_CONSTANTS_GLSL

#include "../common.glsl"

layout(push_constant, std430) uniform PushConstants {
    uint64_t scene_buffer_device_address;
    uint64_t draw_data_buffer_device_address;
    uint64_t entity_buffer_device_address;
    uint64_t vertex_buffer_device_address;
    uint64_t bone_transform_buffer_device_address;
    uint64_t shadow_cascades_buffer_device_address;

    uint shadow_cascade_index;
    uint _pad0;
} push_constants;

#endif
