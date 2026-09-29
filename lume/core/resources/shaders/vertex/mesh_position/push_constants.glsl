#ifndef MESH_POSITION_VERTEX_PUSH_CONSTANTS_GLSL
#define MESH_POSITION_VERTEX_PUSH_CONSTANTS_GLSL

#include "../../common.glsl"

struct MeshPositionVertexShader {
    uint64_t draw_data_buffer_device_address;
    uint64_t entity_buffer_device_address;
    uint64_t vertex_position_buffer_device_address;
    uint64_t shadow_cascades_buffer_device_address;
};

#endif
