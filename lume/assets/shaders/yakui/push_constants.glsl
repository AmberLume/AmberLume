#ifndef PUSH_CONSANTS_GLSL
#define PUSH_CONSANTS_GLSL

#include "../common.glsl"

layout(push_constant, std430) uniform PushConstants {
    uint64_t ui_vertex_buffer_device_address;

    uint texture_index;
    uint render_mode;
} push_constants;

#endif