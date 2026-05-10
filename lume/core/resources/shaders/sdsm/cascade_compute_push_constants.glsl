#ifndef CASCADE_COMPUTE_PUSH_CONSTANTS_GLSL
#define CASCADE_COMPUTE_PUSH_CONSTANTS_GLSL

#extension GL_EXT_buffer_reference : require
#extension GL_EXT_shader_explicit_arithmetic_types_int64 : require

layout(push_constant) uniform PushConstants {
    uint64_t scene_buffer_device_address;
    uint64_t sdsm_buffer_device_address;
    uint64_t culling_view_buffer_device_address;
    uint64_t shadow_cascades_buffer_device_address;

    uint cascade_count;
    uint cascade_view_offset;
    uint shadow_resolution;

    float light_margin;
    float fallback_z_max;
    float split_lambda;
    float shadow_caster_extension;
} push_constants;

#endif
