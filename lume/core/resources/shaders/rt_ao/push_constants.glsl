#ifndef RT_AO_PUSH_CONSTANTS_GLSL
#define RT_AO_PUSH_CONSTANTS_GLSL

#extension GL_EXT_buffer_reference : require
#extension GL_EXT_shader_explicit_arithmetic_types_int64 : require

layout(push_constant, std430) uniform PushConstants {
    uint64_t scene_buffer_device_address;

    uint depth_descriptor_id;
    uint normal_descriptor_id;
    uint ao_storage_id;
    uint width;
    uint height;
    uint tlas_descriptor_id;

    float ao_radius;
    uint sample_count;
    float ao_power;
    uint frame_number;
    uint trace_period;
} push_constants;

#endif
