#version 460
#extension GL_EXT_buffer_reference : require
#extension GL_EXT_shader_explicit_arithmetic_types_int64 : require

layout(buffer_reference, scalar) buffer EntityBuffer {
    mat4 model;
    uint mesh_id;
    uint material_id;
};

layout(buffer_reference, scalar) buffer VertexBuffer {
    vec3 position;
    vec3 normal;
    vec2 uv;
};

layout(set=0, binding=0) uniform Camera {
    mat4 view;
    mat4 proj;
    mat4 view_proj;
    vec4 position;
} camera;

layout(push_constant) uniform PushConstants {
    uint64_t vertex_buffer_address;
    uint64_t index_buffer_address;
    uint64_t entity_buffer_address;
    uint64_t material_buffer_address;
} push_constant;

layout(location=0) out vec3 out_normal;
layout(location=1) out vec2 out_uv;
layout(location=2) flat out uint out_material_id;

void main() {
    EntityBuffer entities = EntityBuffer(push_constant.entity_buffer_address);
    VertexBuffer vertices = VertexBuffer(push_constant.vertex_buffer_address);

    mat4 model = entities[gl_InstanceIndex].model;
    out_material_id = entities[gl_InstanceIndex].material_id;

    vec3 pos = vertices[gl_VertexIndex].position;
    vec3 normal = vertices[gl_VertexIndex].normal;
    vec2 uv = vertices[gl_VertexIndex].uv;

    gl_Position = camera.view_proj * model * vec4(pos, 1.0);
    out_normal = mat3(model) * normal;
    out_uv = uv;
}
