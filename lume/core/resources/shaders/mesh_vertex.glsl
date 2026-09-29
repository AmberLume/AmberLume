#ifndef MESH_VERTEX_GLSL
#define MESH_VERTEX_GLSL

#include "common.glsl"

vec3 vertex_position(VertexPositionBuffer positions, uint index) {
    return vec3(
        positions.data[index].position[0],
        positions.data[index].position[1],
        positions.data[index].position[2]
    );
}

vec3 vertex_normal(VertexNormalTangentBuffer normal_tangents, uint index) {
    return vec3(
        normal_tangents.data[index].normal[0],
        normal_tangents.data[index].normal[1],
        normal_tangents.data[index].normal[2]
    );
}

vec4 vertex_tangent(VertexNormalTangentBuffer normal_tangents, uint index) {
    return vec4(
        normal_tangents.data[index].tangent[0],
        normal_tangents.data[index].tangent[1],
        normal_tangents.data[index].tangent[2],
        normal_tangents.data[index].tangent[3]
    );
}

vec2 vertex_uv(VertexUvBuffer uvs, uint index) {
    return vec2(
        uvs.data[index].uv[0],
        uvs.data[index].uv[1]
    );
}

#endif
