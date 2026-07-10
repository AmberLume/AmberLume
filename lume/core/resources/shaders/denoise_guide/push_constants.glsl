#ifndef DENOISE_GUIDE_PUSH_CONSTANTS_GLSL
#define DENOISE_GUIDE_PUSH_CONSTANTS_GLSL

layout(push_constant, std430) uniform PushConstants {
    mat4 inverse_view_projection;

    vec4 camera_position;

    uint depth_descriptor_id;
    uint normal_descriptor_id;
    uint guide_storage_id;
    uint width;
    uint height;
} push_constants;

#endif
