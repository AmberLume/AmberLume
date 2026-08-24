#ifndef MESH_VERTEX_GLSL
#define MESH_VERTEX_GLSL

#include "common.glsl"

vec3 mesh_vertex_position(MeshVertex vertex) {
    return vec3(vertex.position[0], vertex.position[1], vertex.position[2]);
}

vec3 mesh_vertex_normal(MeshVertex vertex) {
    return vec3(vertex.normal[0], vertex.normal[1], vertex.normal[2]);
}

#endif
