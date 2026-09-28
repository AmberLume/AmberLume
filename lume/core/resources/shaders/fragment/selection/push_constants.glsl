#ifndef SELECTION_FRAGMENT_PUSH_CONSTANTS_GLSL
#define SELECTION_FRAGMENT_PUSH_CONSTANTS_GLSL

#include "../../common.glsl"

struct SelectionFragmentShader {
    uint64_t camera_buffer_device_address;
    uint64_t entity_outline_buffer_device_address;

    vec2 entity_id_texel_scale;

    uint entity_id_texture;
    uint mask_texture;

    int radius;
    int mask_scale;
};

layout(push_constant, std430) uniform PushConstants {
    SelectionFragmentShader fragment;
} push_constants;

#endif
