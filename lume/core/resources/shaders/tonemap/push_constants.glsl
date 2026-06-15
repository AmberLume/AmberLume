#ifndef TONEMAP_PUSH_CONSTANTS_GLSL
#define TONEMAP_PUSH_CONSTANTS_GLSL

layout(push_constant) uniform PushConstants {
    uint input_texture;
    float exposure;
    uint hdr;
    float paper_white;
    uint bloom_texture;
    float bloom_intensity;
    float sharpness;
} push_constants;

#endif
