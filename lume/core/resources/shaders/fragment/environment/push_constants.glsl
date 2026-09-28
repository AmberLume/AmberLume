#ifndef ENVIRONMENT_FRAGMENT_PUSH_CONSTANTS_GLSL
#define ENVIRONMENT_FRAGMENT_PUSH_CONSTANTS_GLSL

#include "../../common.glsl"

struct EnvironmentFragmentShader {
    uint64_t scene_buffer_device_address;
    uint64_t camera_buffer_device_address;
};

layout(push_constant, std430) uniform PushConstants {
    EnvironmentFragmentShader fragment;
} push_constants;

#endif
