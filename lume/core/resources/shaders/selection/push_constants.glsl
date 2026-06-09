#ifndef SELECTION_PUSH_CONSTANTS_GLSL
#define SELECTION_PUSH_CONSTANTS_GLSL

layout(push_constant) uniform PushConstants {
    vec4 color;

    uint entity_id_texture;
    uint selected_entity;

    float stripe_width;
} push_constants;

#endif
