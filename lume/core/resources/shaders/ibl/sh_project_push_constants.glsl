#ifndef SH_PROJECT_PUSH_CONSTANTS_GLSL
#define SH_PROJECT_PUSH_CONSTANTS_GLSL

layout(push_constant, std430) uniform PushConstants {
    uint64_t scene_buffer_device_address;
} push_constants;

#endif
