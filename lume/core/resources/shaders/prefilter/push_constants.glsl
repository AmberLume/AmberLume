#ifndef PREFILTER_PUSH_CONSTANTS_GLSL
#define PREFILTER_PUSH_CONSTANTS_GLSL

layout(push_constant) uniform PushConstants {
    uint depth_descriptor_id;
    uint width;
    uint height;
    uint mip_count;

    uint mip0_id;
    uint mip1_id;
    uint mip2_id;
    uint mip3_id;
    uint mip4_id;
} push_constants;

#endif
