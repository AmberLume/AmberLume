#ifndef FSR2_ACCUMULATE_PUSH_CONSTANTS_GLSL
#define FSR2_ACCUMULATE_PUSH_CONSTANTS_GLSL

layout(push_constant) uniform PushConstants {
    uint scene_color_texture;
    uint velocity_texture;
    uint history_prev_texture;
    uint history_curr_storage;
    uint history_valid;
    uint display_width;
    uint display_height;
} push_constants;

#endif
