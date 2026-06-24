#ifndef DEBUG_LAYER_PUSH_CONSTANTS_GLSL
#define DEBUG_LAYER_PUSH_CONSTANTS_GLSL

const uint DEBUG_LAYER_VELOCITY = 1u;
const uint DEBUG_LAYER_NORMAL = 2u;
const uint DEBUG_LAYER_GTAO = 3u;
const uint DEBUG_LAYER_SH_IRRADIANCE = 4u;

layout(push_constant) uniform PushConstants {
    mat4 inverse_view_projection;

    uint texture_index;
    uint layer_kind;
} push_constants;

#endif
