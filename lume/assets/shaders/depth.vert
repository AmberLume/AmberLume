#version 460
#extension GL_EXT_buffer_reference : require
#extension GL_EXT_buffer_reference_uvec2 : require

struct Vertex {
    vec3 position;
    vec3 normal;
    vec2 uv;
};

layout(buffer_reference, std430) readonly buffer VertexBuffer {
    Vertex vertices[];
};

layout(push_constant) uniform PushConstants {
    mat4 view_projection;
    uvec2 vertex_buffer_address;
} push_constants;

void main() {
    VertexBuffer vertex_buffer = VertexBuffer(push_constants.vertex_buffer_address);
    vec3 position = vertex_buffer.vertices[gl_VertexIndex].position;

    gl_Position = push_constants.view_projection * vec4(position, 1.0);
}
