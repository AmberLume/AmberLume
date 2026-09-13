#ifndef PUSH_CONSTANTS_GLSL
#define PUSH_CONSTANTS_GLSL

#include "../common.glsl"

layout(push_constant, std430) uniform PushConstants {
    uint64_t skin_cache_instance_buffer_device_address;
    uint64_t entity_buffer_device_address;
    uint64_t bone_transform_buffer_device_address;
    uint64_t mesh_vertex_buffer_device_address;
    uint64_t mesh_vertex_attribute_buffer_device_address;
    uint64_t mesh_vertex_skin_buffer_device_address;
    uint64_t skin_cache_vertex_buffer_device_address;
    uint64_t skin_cache_vertex_attribute_buffer_device_address;

    uint instance_count;
    uint vertex_count;
} push_constants;

#endif
