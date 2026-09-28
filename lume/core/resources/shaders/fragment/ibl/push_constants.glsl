#ifndef IBL_FRAGMENT_PUSH_CONSTANTS_GLSL
#define IBL_FRAGMENT_PUSH_CONSTANTS_GLSL

#include "../../common.glsl"

struct ShProjectFragmentShader {
    uint64_t scene_buffer_device_address;
};

layout(push_constant, std430) uniform PushConstants {
    ShProjectFragmentShader fragment;
} push_constants;

#endif
