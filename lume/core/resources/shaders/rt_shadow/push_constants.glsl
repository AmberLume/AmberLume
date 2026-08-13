#ifndef RT_SHADOW_PUSH_CONSTANTS_GLSL
#define RT_SHADOW_PUSH_CONSTANTS_GLSL

#extension GL_EXT_buffer_reference : require
#extension GL_EXT_shader_explicit_arithmetic_types_int64 : require

layout(push_constant, std430) uniform PushConstants {
    uint64_t scene_buffer_device_address;

    uint depth_descriptor_id;
    uint normal_descriptor_id;
    uint visibility_storage_id;
    uint tlas_descriptor_id;

    float sun_angular_radius;
    uint sample_count;
    uint frame_number;
} push_constants;

#endif
