#ifndef YAKUI_FRAGMENT_PUSH_CONSTANTS_GLSL
#define YAKUI_FRAGMENT_PUSH_CONSTANTS_GLSL

#include "../../vertex/yakui/push_constants.glsl"

struct YakuiFragmentShader {
    uint texture_index;
    uint render_mode;
};

layout(push_constant, std430) uniform PushConstants {
    YakuiVertexShader vertex;
    YakuiFragmentShader fragment;
} push_constants;

#endif
