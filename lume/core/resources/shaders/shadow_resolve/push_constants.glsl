#ifndef SHADOW_RESOLVE_PUSH_CONSTANTS_GLSL
#define SHADOW_RESOLVE_PUSH_CONSTANTS_GLSL

layout(push_constant, std430) uniform PushConstants {
    mat4 inverse_view_projection;

    uint64_t scene_buffer_device_address;
    uint64_t shadow_cascades_buffer_device_address;

    uint depth_descriptor_id;
    uint normal_descriptor_id;
    uint shadow_array_descriptor_id;
    uint output_storage_id;

    uint shadow_pcf_sample_count;
    uint frame_index;

    float shadow_bias;
    float shadow_normal_bias;
    float shadow_pcf_world_radius;
    float shadow_cascade_blend_range;
} push_constants;

#endif
