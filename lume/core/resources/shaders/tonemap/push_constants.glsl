#ifndef TONEMAP_PUSH_CONSTANTS_GLSL
#define TONEMAP_PUSH_CONSTANTS_GLSL

layout(push_constant) uniform PushConstants {
    uint input_texture;
    float exposure;
} push_constants;

#endif
