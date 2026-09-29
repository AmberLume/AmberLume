#ifndef TERRAIN_POINTS_VERTEX_PUSH_CONSTANTS_GLSL
#define TERRAIN_POINTS_VERTEX_PUSH_CONSTANTS_GLSL

#include "../../common.glsl"

struct TerrainPointsVertexShader {
    uint64_t camera_buffer_device_address;
    uint64_t chunk_buffer_device_address;
    uint64_t vertex_position_buffer_device_address;
    uint64_t mesh_buffer_device_address;
    uint64_t submesh_buffer_device_address;

    uint node_count;
    float point_size;
    float viewport_height;

    uint _pad0;
};

#endif
