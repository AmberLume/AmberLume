#ifndef GTAO_DEPTH_MIP_PUSH_CONSTANTS_GLSL
#define GTAO_DEPTH_MIP_PUSH_CONSTANTS_GLSL

layout(push_constant, std430) uniform PushConstants {
    uint source_descriptor_id;
    uint view_z_storage_id;
    uint width;
    uint height;
    uint source_width;
    uint source_height;

    float radius;
} push_constants;

#endif
