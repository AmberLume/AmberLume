#version 460

#extension GL_ARB_shader_draw_parameters : enable

#include "../common.glsl"
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
    Vertex vertex = VertexBuffer(push_constants.vertex_buffer_device_address).data[gl_VertexIndex];

    uint bone_index_0 =  vertex.bone_indices[0];
    uint bone_index_1 = (vertex.bone_indices[0] >> 16);
    uint bone_index_2 =  vertex.bone_indices[1];
    uint bone_index_3 = (vertex.bone_indices[1] >> 16);

    mat4 skin_matrix;
    if (entity.is_skinned == 1) {
        BoneTransformBuffer transforms = BoneTransformBuffer(push_constants.bone_transform_buffer_device_address);

        uint base = entity.bone_transform_offset;

        skin_matrix =
            transforms.data[base + bone_index_0].transform * vertex.bone_weights[0] +
            transforms.data[base + bone_index_1].transform * vertex.bone_weights[1] +
            transforms.data[base + bone_index_2].transform * vertex.bone_weights[2] +
            transforms.data[base + bone_index_3].transform * vertex.bone_weights[3];
    } else {
        skin_matrix = entity.transform_matrix;
    }

    vec4 world_position = skin_matrix * vec4(vertex.position[0], vertex.position[1], vertex.position[2], 1.0);
    gl_Position = scene_buffer.data.main_camera.projection_matrix * world_position;

    mat3 mesh_mat = mat3(entity.transform_matrix);

    vec3 T = normalize(mesh_mat * vec3(vertex.tangent[0], vertex.tangent[1], vertex.tangent[2]));
    vec3 N = normalize(mesh_mat * vec3(vertex.normal[0], vertex.normal[1], vertex.normal[2]));

    T = normalize(T - dot(T, N) * N);

    vec3 B = cross(N, T) * vertex.tangent[3];

    out_TBN = mat3(T, B, N);
    uv = vec2(vertex.uv[0], vertex.uv[1]);
    world_pos = world_position.xyz;
}
