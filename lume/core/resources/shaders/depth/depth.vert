#version 460

#include "../common.glsl"
#include "../mesh_vertex.glsl"
#include "../skinning.glsl"
#include "push_constants.glsl"

layout(location = 0) out vec3 world_normal;
layout(location = 1) out vec4 current_clip;
layout(location = 2) out vec4 previous_clip;

void main() {
    SceneBuffer scene_buffer = SceneBuffer(push_constants.scene_buffer_device_address);
    DrawData draw_data = DrawDataBuffer(push_constants.draw_data_buffer_device_address).data[gl_InstanceIndex];
    Entity entity = EntityBuffer(push_constants.entity_buffer_device_address).data[draw_data.entity_index];
    EntityMotion entity_motion = EntityMotionBuffer(push_constants.entity_motion_buffer_device_address).data[draw_data.entity_index];
    Submesh submesh = SubmeshBuffer(push_constants.submesh_buffer_device_address).data[draw_data.submesh_index];

    MeshVertex vertex = MeshVertexBuffer(push_constants.mesh_vertex_buffer_device_address).data[gl_VertexIndex];

    uint skin_index = submesh.vertex_skin_offset + uint(gl_VertexIndex) - submesh.vertex_offset;

    vec4 local_position = vec4(mesh_vertex_position(vertex), 1.0);

    mat4 skin_matrix = compute_skin_matrix(
        entity.transform_matrix,
        entity.bone_transform_offset,
        skin_index,
        push_constants.mesh_vertex_skin_buffer_device_address,
        push_constants.bone_transform_buffer_device_address
    );
    mat3 normal_matrix = mat3(transpose(inverse(skin_matrix)));
    vec4 world_position = skin_matrix * local_position;

    mat4 previous_skin_matrix = compute_skin_matrix(
        entity_motion.previous_transform_matrix,
        entity.bone_transform_offset,
        skin_index,
        push_constants.mesh_vertex_skin_buffer_device_address,
        push_constants.bone_transform_buffer_device_address
    );
    vec4 previous_world_position = previous_skin_matrix * local_position;

    world_normal = normalize(normal_matrix * mesh_vertex_normal(vertex));

    current_clip = scene_buffer.data.main_camera.view_projection * world_position;
    previous_clip = scene_buffer.data.main_camera.previous_view_projection * previous_world_position;

    gl_Position = scene_buffer.data.main_camera.jittered_view_projection * world_position;
}
