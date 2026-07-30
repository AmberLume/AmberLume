#ifndef RT_SHADOW_PUSH_CONSTANTS_GLSL
#define RT_SHADOW_PUSH_CONSTANTS_GLSL

layout(push_constant, std430) uniform PushConstants {
    mat4 inverse_view_projection;

    vec4 sun_direction;

    uint depth_descriptor_id;
    uint normal_descriptor_id;
    uint visibility_storage_id;
    uint tlas_descriptor_id;

    float sun_angular_radius;
    uint sample_count;
    uint frame_number;
} push_constants;

#endif
