#ifndef MESH_SURFACE_VERTEX_PUSH_CONSTANTS_GLSL
#define MESH_SURFACE_VERTEX_PUSH_CONSTANTS_GLSL

#include "../../common.glsl"

struct MeshSurfaceVertexShader {
    uint64_t camera_buffer_device_address;
    uint64_t draw_data_buffer_device_address;
    uint64_t entity_buffer_device_address;
    uint64_t submesh_buffer_device_address;
};

#endif
