#ifndef UPSAMPLE_FRAGMENT_PUSH_CONSTANTS_GLSL
#define UPSAMPLE_FRAGMENT_PUSH_CONSTANTS_GLSL

struct UpsampleFragmentShader {
    uint src_texture;
};

layout(push_constant, std430) uniform PushConstants {
    UpsampleFragmentShader fragment;
} push_constants;

#endif
