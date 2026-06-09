#ifndef TONEMAP_PUSH_CONSTANTS_GLSL
#define TONEMAP_PUSH_CONSTANTS_GLSL

layout(push_constant) uniform PushConstants {
    uint input_texture;
    float exposure;
    uint hdr;
    float paper_white;
} push_constants;

#endif
