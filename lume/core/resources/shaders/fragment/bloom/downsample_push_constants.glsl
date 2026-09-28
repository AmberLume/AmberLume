#ifndef DOWNSAMPLE_FRAGMENT_PUSH_CONSTANTS_GLSL
#define DOWNSAMPLE_FRAGMENT_PUSH_CONSTANTS_GLSL

struct DownsampleFragmentShader {
    uint src_texture;
    uint karis;
    float threshold;
};

layout(push_constant, std430) uniform PushConstants {
    DownsampleFragmentShader fragment;
} push_constants;

#endif
