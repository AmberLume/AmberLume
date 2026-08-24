#ifndef PUSH_CONSTANTS_GLSL
#define PUSH_CONSTANTS_GLSL

#include "../common.glsl"

layout(push_constant, std430) uniform PushConstants {
    uint64_t scene_buffer_device_address;
    uint64_t draw_data_buffer_device_address;
    uint64_t mesh_vertex_buffer_device_address;
    uint64_t mesh_vertex_attribute_buffer_device_address;
    uint64_t mesh_vertex_skin_buffer_device_address;
    uint64_t entity_buffer_device_address;
    uint64_t submesh_buffer_device_address;
    uint64_t material_buffer_device_address;
    uint64_t bone_transform_buffer_device_address;

    uint sh_descriptor_id;
    uint brdf_lut_descriptor_id;
} push_constants;

#endif
