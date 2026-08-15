#ifndef GTAO_DEPTH_PUSH_CONSTANTS_GLSL
#define GTAO_DEPTH_PUSH_CONSTANTS_GLSL

#extension GL_EXT_buffer_reference : require
#extension GL_EXT_shader_explicit_arithmetic_types_int64 : require

layout(push_constant, std430) uniform PushConstants {
    uint64_t scene_buffer_device_address;

    uint depth_descriptor_id;
    uint view_z_storage_id;
    uint width;
    uint height;
} push_constants;

#endif
