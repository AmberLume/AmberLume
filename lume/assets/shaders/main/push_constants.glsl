#ifndef PUSH_CONSANTS_GLSL
#define PUSH_CONSANTS_GLSL

#include "../common.glsl"

layout(push_constant, std430) uniform PushConstants {
    mat4 projection_matrix;
    vec3 light_direction;
    uint _pad0;
    vec3 camera_position;
    uint _pad1;

    uint64_t draw_data_buffer_device_address;
    uint64_t vertex_buffer_device_address;
    uint64_t entity_buffer_device_address;
    uint64_t submesh_buffer_device_address;
    uint64_t material_buffer_device_address;

    uint shadow_mask_descriptor_id;

    uint _pad2;
} push_constants;

#endif
