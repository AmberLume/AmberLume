#ifndef HIZ_PUSH_CONSTANTS_GLSL
#define HIZ_PUSH_CONSTANTS_GLSL

#extension GL_EXT_buffer_reference : require
#extension GL_EXT_shader_explicit_arithmetic_types_int64 : require

#define HIZ_MAX_MIPS 16

layout(buffer_reference, std430) coherent buffer HiZCounterBuffer {
    uint counter[];
};

layout(push_constant) uniform PushConstants {
    uint64_t counter_buffer_address;

    uint depth_descriptor_id;
    uint mip_count;
    uint width;
    uint height;

    uint storage_ids[HIZ_MAX_MIPS];
} push_constants;

#endif
