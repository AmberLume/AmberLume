#ifndef PUSH_CONSTANTS_GLSL
#define PUSH_CONSTANTS_GLSL

#include "../common.glsl"

layout(push_constant, std430) uniform PushConstants {
    uint64_t indirect_source_buffer_device_address;
    uint64_t indirect_sorted_buffer_device_address;
    uint64_t draw_count_buffer_device_address;
    uint64_t draw_data_buffer_device_address;
    uint64_t statistics_buffer_device_address;

    uint count_index;
    uint source_offset;
    uint target_offset;
    uint capacity;
} push_constants;

#endif
