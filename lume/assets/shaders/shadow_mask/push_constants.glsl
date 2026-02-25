#ifndef PUSH_CONSANTS_GLSL
#define PUSH_CONSANTS_GLSL

#include "../common.glsl"

layout(push_constant, std430) uniform PushConstants {
    uint64_t shadow_buffer_device_address;

    uint cascade_count;
    float camera_near;
    float camera_far;
    float bias;
    int pcf_radius;

    uint depth_descriptor_id;
    uint global_shadow_descriptor_id;
} push_constants;

#endif
