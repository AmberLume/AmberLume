#ifndef DEBUG_LAYER_PUSH_CONSTANTS_GLSL
#define DEBUG_LAYER_PUSH_CONSTANTS_GLSL

layout(push_constant) uniform PushConstants {
    uint texture_index;
    uint layer_kind;
} push_constants;

#endif
