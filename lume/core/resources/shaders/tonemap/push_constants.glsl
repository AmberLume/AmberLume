#ifndef TONEMAP_PUSH_CONSTANTS_GLSL
#define TONEMAP_PUSH_CONSTANTS_GLSL

layout(push_constant) uniform PushConstants {
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
} push_constants;

#endif
