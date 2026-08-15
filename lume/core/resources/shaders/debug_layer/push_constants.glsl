#ifndef DEBUG_LAYER_PUSH_CONSTANTS_GLSL
#define DEBUG_LAYER_PUSH_CONSTANTS_GLSL

#extension GL_EXT_buffer_reference : require
#extension GL_EXT_shader_explicit_arithmetic_types_int64 : require

const uint DEBUG_LAYER_VELOCITY = 1u;
const uint DEBUG_LAYER_NORMAL = 2u;
const uint DEBUG_LAYER_GTAO = 3u;
const uint DEBUG_LAYER_SH_IRRADIANCE = 4u;
const uint DEBUG_LAYER_HIZ_NEAR = 5u;
const uint DEBUG_LAYER_HIZ_FAR = 6u;
const uint DEBUG_LAYER_SHADOW = 7u;
const uint DEBUG_LAYER_AO_HISTORY = 8u;
const uint DEBUG_LAYER_AO_DENOISED = 9u;

layout(push_constant) uniform PushConstants {
    uint64_t scene_buffer_device_address;

    uint texture_index;
    uint layer_kind;
    uint shadow_colored;

    float denoise_history;
} push_constants;

#endif
