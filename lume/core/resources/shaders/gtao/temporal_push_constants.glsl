#ifndef GTAO_TEMPORAL_PUSH_CONSTANTS_GLSL
#define GTAO_TEMPORAL_PUSH_CONSTANTS_GLSL

layout(push_constant) uniform PushConstants {
    uint gtao_texture;
    uint velocity_texture;
    uint history_prev_texture;
    uint history_curr_storage;
    uint resolved_storage;
    uint history_valid;
    uint width;
    uint height;
} push_constants;

#endif
