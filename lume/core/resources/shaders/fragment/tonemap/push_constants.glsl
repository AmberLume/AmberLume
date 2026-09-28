#ifndef TONEMAP_FRAGMENT_PUSH_CONSTANTS_GLSL
#define TONEMAP_FRAGMENT_PUSH_CONSTANTS_GLSL

struct TonemapFragmentShader {
    uint input_texture;
    float exposure;
    float saturation;
    float contrast;
    float offset[3];
    float gamma[3];
    float gain[3];
    uint hdr;
    float paper_white;
    float display_peak;
    uint bloom_texture;
    float bloom_intensity;
    float sharpness;
};

layout(push_constant, std430) uniform PushConstants {
    TonemapFragmentShader fragment;
} push_constants;

#endif
