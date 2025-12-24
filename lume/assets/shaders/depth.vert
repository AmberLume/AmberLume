#version 460
#extension GL_EXT_buffer_reference : require
#extension GL_EXT_shader_explicit_arithmetic_types_int64 : require

struct Vertex {
    vec3 position;
    float _pad0;
    vec3 normal;
    float _pad1;
    vec2 uv;
    vec2 _pad2;
};

layout(buffer_reference, std430, buffer_reference_align = 16) readonly buffer VertexBuffer {
    Vertex vertices[];
};

layout(push_constant, std430) uniform PushConstants {
    mat4 view_projection;
    uint64_t vertex_buffer_address;
} push_constants;

void main() {
    VertexBuffer vertex_buffer = VertexBuffer(push_constants.vertex_buffer_address);

    vec3 position = vertex_buffer.vertices[gl_VertexIndex].position;

    gl_Position = push_constants.view_projection * vec4(position, 1.0);
}
