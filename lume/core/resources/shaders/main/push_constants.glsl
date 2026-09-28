#ifndef PUSH_CONSTANTS_GLSL
#define PUSH_CONSTANTS_GLSL

#include "../common.glsl"
#include "../vertex/mesh_shading_vertex_shader.glsl"
#include "main_fragment_shader.glsl"

layout(push_constant, std430) uniform PushConstants {
    MeshShadingVertexShader vertex;
    MainFragmentShader fragment;
} push_constants;

#endif
