#ifndef BLOOM_PUSH_CONSTANTS_GLSL
#define BLOOM_PUSH_CONSTANTS_GLSL

layout(push_constant) uniform PushConstants {
    uint src_texture;
    uint karis;
    float threshold;
} push_constants;

#endif
