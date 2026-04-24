#ifndef PUSH_CONSTANTS_GLSL
#define PUSH_CONSTANTS_GLSL

#include "../common.glsl"

layout(push_constant) uniform PushConstants {
    uint64_t skinning_instance_buffer_device_address;
    uint64_t animation_buffer_device_address;
    uint64_t animation_frame_buffer_device_address;
    uint64_t skeleton_buffer_device_address;
    uint64_t skeleton_bone_buffer_device_address;
    uint64_t bone_transform_buffer_device_address;
    uint instance_count;
} push_constants;

#endif