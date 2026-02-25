#ifndef PUSH_CONSANTS_GLSL
#define PUSH_CONSANTS_GLSL

#include "../common.glsl"

layout(push_constant, std430) uniform PushConstants {
    mat4 projection_matrix;

    uint64_t draw_data_buffer_device_address;
    uint64_t vertex_buffer_device_address;
    uint64_t entity_buffer_device_address;
    uint64_t submesh_buffer_device_address;
    uint64_t material_buffer_device_address;

    uint shadow_mask_descriptor_id;
} push_constants;

#endif
