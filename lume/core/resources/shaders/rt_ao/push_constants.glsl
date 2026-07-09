#ifndef RT_AO_PUSH_CONSTANTS_GLSL
#define RT_AO_PUSH_CONSTANTS_GLSL

layout(push_constant, std430) uniform PushConstants {
    mat4 inverse_view_projection;

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
} push_constants;

#endif
