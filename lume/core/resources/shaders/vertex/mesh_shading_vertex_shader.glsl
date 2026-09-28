#ifndef MESH_SHADING_VERTEX_SHADER_GLSL
#define MESH_SHADING_VERTEX_SHADER_GLSL

#include "../common.glsl"

struct MeshShadingVertexShader {
    uint64_t camera_buffer_device_address;
    uint64_t draw_data_buffer_device_address;
    uint64_t entity_buffer_device_address;
    uint64_t submesh_buffer_device_address;
};

#endif
