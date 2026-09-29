#ifndef PUSH_CONSTANTS_GLSL
#define PUSH_CONSTANTS_GLSL

#include "../common.glsl"

layout(push_constant) uniform PushConstants {
    uint64_t skin_target_buffer_device_address;
    uint64_t skinning_instance_buffer_device_address;
    uint64_t bone_transform_buffer_device_address;
    uint64_t vertex_position_buffer_device_address;
    uint64_t vertex_normal_tangent_buffer_device_address;
    uint64_t vertex_skin_buffer_device_address;

    uint target_count;
    uint vertex_count;
} push_constants;

#endif
