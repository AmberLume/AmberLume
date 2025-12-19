#version 450
#extension GL_EXT_buffer_reference : require
#extension GL_EXT_scalar_block_layout : require

struct Vertex {
    vec3 position;
    vec3 normal;
    vec2 uv;
};

layout(buffer_reference, scalar) readonly buffer VertexBuffer {
    Vertex vertices[];
};

layout(push_constant) uniform PushConstants {
    mat4 view_projection;
    VertexBuffer vertex_buffer;
} push;

void main() {
    Vertex vertex = push.vertex_buffer.vertices[gl_VertexIndex];
    gl_Position = push.view_projection * vec4(vertex.position, 1.0);
}
