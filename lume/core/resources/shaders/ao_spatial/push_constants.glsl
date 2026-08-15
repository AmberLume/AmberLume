#ifndef AO_SPATIAL_PUSH_CONSTANTS_GLSL
#define AO_SPATIAL_PUSH_CONSTANTS_GLSL

#extension GL_EXT_buffer_reference : require
#extension GL_EXT_shader_explicit_arithmetic_types_int64 : require

layout(push_constant, std430) uniform PushConstants {
    uint64_t scene_buffer_device_address;

    uint noisy_descriptor_id;
    uint guide_descriptor_id;
    uint ao_storage_id;
    uint width;
    uint height;

    float plane_sensitivity;
    float normal_threshold;
    float blur_radius;

    uint frame_number;
} push_constants;

#endif
