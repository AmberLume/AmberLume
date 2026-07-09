#ifndef PUSH_CONSTANTS_GLSL
#define PUSH_CONSTANTS_GLSL

#include "../common.glsl"

layout(push_constant) uniform PushConstants {
    uint64_t entity_buffer_device_address;
    uint64_t blas_address_buffer_device_address;
    uint64_t instance_buffer_device_address;

    uint entity_count;
} push_constants;

#endif
