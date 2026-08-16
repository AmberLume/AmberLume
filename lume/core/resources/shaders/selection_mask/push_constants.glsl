#ifndef SELECTION_MASK_PUSH_CONSTANTS_GLSL
#define SELECTION_MASK_PUSH_CONSTANTS_GLSL

#extension GL_EXT_buffer_reference : require
#extension GL_EXT_shader_explicit_arithmetic_types_int64 : require

layout(push_constant, std430) uniform PushConstants {
    uint64_t entity_buffer_device_address;

    uint entity_id_texture;
    uint mask_storage_id;
    uint width;
    uint height;

    int mask_scale;
} push_constants;

#endif
