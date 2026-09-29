#version 460

#extension GL_ARB_shader_draw_parameters : enable

#include "../../common.glsl"
#include "../../projection.glsl"
#include "../../mesh_vertex.glsl"
#include "push_constants.glsl"

layout(push_constant, std430) uniform PushConstants {
    MeshSurfaceVertexShader vertex;
} push_constants;

layout(location = 0) out mat3 out_TBN;
layout(location = 3) out vec2 uv;
layout(location = 4) out flat uint draw_id;
layout(location = 5) out vec3 world_pos;

void main() {
    draw_id = gl_InstanceIndex;

    CameraBuffer camera = CameraBuffer(push_constants.vertex.camera_buffer_device_address);
    DrawData draw_data = DrawDataBuffer(push_constants.vertex.draw_data_buffer_device_address).data[draw_id];
    Entity entity = EntityBuffer(push_constants.vertex.entity_buffer_device_address).data[draw_data.entity_index];
    Submesh submesh = SubmeshBuffer(push_constants.vertex.submesh_buffer_device_address).data[draw_data.submesh_index];

    VertexPositionBuffer positions = VertexPositionBuffer(push_constants.vertex.vertex_position_buffer_device_address);
    VertexNormalTangentBuffer normal_tangents = VertexNormalTangentBuffer(push_constants.vertex.vertex_normal_tangent_buffer_device_address);
    VertexUvBuffer uvs = VertexUvBuffer(push_constants.vertex.vertex_uv_buffer_device_address);

    uint vertex_index = uint(gl_VertexIndex);
    uint local_vertex_index = vertex_index - submesh.vertex_offset;

    vec4 tangent = vertex_tangent(normal_tangents, vertex_index);

    mat3 normal_mat  = mat3(transpose(inverse(entity.transform_matrix)));
    vec4 world_position = entity.transform_matrix * vec4(vertex_position(positions, vertex_index), 1.0);

    vec4 clip_position = camera.view_projection * world_position;

    gl_Position = jitter_clip_position(clip_position, camera.jitter);

    vec3 T = normalize(normal_mat * tangent.xyz);
    vec3 N = normalize(normal_mat * vertex_normal(normal_tangents, vertex_index));

    T = normalize(T - dot(T, N) * N);

    vec3 B = cross(N, T) * tangent.w;

    out_TBN = mat3(T, B, N);
    uv = vertex_uv(uvs, submesh.uv_offset + local_vertex_index);
    world_pos = world_position.xyz;
}
