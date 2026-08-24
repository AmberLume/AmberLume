#ifndef SELECTION_PUSH_CONSTANTS_GLSL
#define SELECTION_PUSH_CONSTANTS_GLSL

#extension GL_EXT_buffer_reference : require
#extension GL_EXT_shader_explicit_arithmetic_types_int64 : require

layout(push_constant, std430) uniform PushConstants {
    uint64_t scene_buffer_device_address;
    uint64_t entity_outline_buffer_device_address;

    vec2 entity_id_texel_scale;

    uint entity_id_texture;
    uint mask_texture;

    int radius;
    int mask_scale;
} push_constants;

#endif
