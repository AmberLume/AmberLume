#ifndef PREFILTER_PUSH_CONSTANTS_GLSL
#define PREFILTER_PUSH_CONSTANTS_GLSL

#extension GL_EXT_buffer_reference : require
#extension GL_EXT_shader_explicit_arithmetic_types_int64 : require

layout(push_constant) uniform PushConstants {
    uint64_t scene_buffer_device_address;

    uint depth_descriptor_id;
    uint width;
    uint height;
    uint mip_count;

    uint mip0_id;
    uint mip1_id;
    uint mip2_id;
    uint mip3_id;
    uint mip4_id;
} push_constants;

#endif
