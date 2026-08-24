#version 460

#extension GL_ARB_shader_draw_parameters : enable

#include "../common.glsl"
#include "../mesh_vertex.glsl"
#include "../skinning.glsl"
#include "push_constants.glsl"

layout(location = 0) out mat3 out_TBN;
layout(location = 3) out vec2 uv;
layout(location = 4) out flat uint draw_id;
layout(location = 5) out vec3 world_pos;

void main() {
    draw_id = gl_InstanceIndex;

    SceneBuffer scene_buffer = SceneBuffer(push_constants.scene_buffer_device_address);
    DrawData draw_data = DrawDataBuffer(push_constants.draw_data_buffer_device_address).data[draw_id];
    Entity entity = EntityBuffer(push_constants.entity_buffer_device_address).data[draw_data.entity_index];
    Submesh submesh = SubmeshBuffer(push_constants.submesh_buffer_device_address).data[draw_data.submesh_index];

    uint local_vertex_index = uint(gl_VertexIndex) - submesh.vertex_offset;

    MeshVertex vertex = MeshVertexBuffer(push_constants.mesh_vertex_buffer_device_address).data[gl_VertexIndex];
    MeshVertexAttribute vertex_attribute = MeshVertexAttributeBuffer(push_constants.mesh_vertex_attribute_buffer_device_address)
        .data[submesh.vertex_attribute_offset + local_vertex_index];

    mat4 skin_matrix = compute_skin_matrix(
        entity.transform_matrix,
        entity.bone_transform_offset,
        submesh.vertex_skin_offset + local_vertex_index,
        push_constants.mesh_vertex_skin_buffer_device_address,
        push_constants.bone_transform_buffer_device_address
    );
    mat3 normal_mat  = mat3(transpose(inverse(skin_matrix)));
    vec4 world_position = skin_matrix * vec4(mesh_vertex_position(vertex), 1.0);

    gl_Position = scene_buffer.data.main_camera.jittered_view_projection * world_position;

    vec3 T = normalize(normal_mat * vec3(vertex_attribute.tangent[0], vertex_attribute.tangent[1], vertex_attribute.tangent[2]));
    vec3 N = normalize(normal_mat * mesh_vertex_normal(vertex));

    T = normalize(T - dot(T, N) * N);

    vec3 B = cross(N, T) * vertex_attribute.tangent[3];

    out_TBN = mat3(T, B, N);
    uv = vec2(vertex_attribute.uv[0], vertex_attribute.uv[1]);
    world_pos = world_position.xyz;
}
