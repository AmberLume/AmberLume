#ifndef DEPTH_REDUCE_PUSH_CONSTANTS_GLSL
#define DEPTH_REDUCE_PUSH_CONSTANTS_GLSL

#extension GL_EXT_buffer_reference : require
#extension GL_EXT_shader_explicit_arithmetic_types_int64 : require

layout(push_constant) uniform PushConstants {
    uint64_t result_buffer_device_address;

    uint depth_descriptor_id;
    uint depth_width;
    uint depth_height;
    uint stride;
} push_constants;

#endif
