#ifndef GTAO_TEMPORAL_PUSH_CONSTANTS_GLSL
#define GTAO_TEMPORAL_PUSH_CONSTANTS_GLSL

layout(push_constant, std430) uniform PushConstants {
    uint guide_curr_tex;
    uint guide_prev_tex;
    uint velocity_tex;
    uint noisy_tex;
    uint signal_prev_tex;
    uint signal_storage;

    uint width;
    uint height;
    uint history_valid;
    uint history_frames;
    uint frame_number;
    uint trace_period;
    uint variance_clamp;

    float tau_z;
    float tau_n;
} push_constants;

#endif
