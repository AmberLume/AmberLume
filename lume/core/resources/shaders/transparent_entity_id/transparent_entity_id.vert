#version 460

#include "../common.glsl"
#include "../skinning.glsl"
#include "push_constants.glsl"

layout(location = 0) out flat uint entity_index;
layout(location = 1) out vec4 current_clip;
layout(location = 2) out vec4 previous_clip;

void main() {
    SceneBuffer scene_buffer = SceneBuffer(push_constants.scene_buffer_device_address);
    DrawData draw_data = DrawDataBuffer(push_constants.draw_data_buffer_device_address).data[gl_InstanceIndex];
    Entity entity = EntityBuffer(push_constants.entity_buffer_device_address).data[draw_data.entity_index];
    Vertex vertex = VertexBuffer(push_constants.vertex_buffer_device_address).data[gl_VertexIndex];

    entity_index = draw_data.entity_index;

    vec4 local_position = vec4(vertex.position[0], vertex.position[1], vertex.position[2], 1.0);

    mat4 skin_matrix = compute_skin_matrix(entity, vertex, push_constants.bone_transform_buffer_device_address);
    vec4 world_position = skin_matrix * local_position;

    mat4 previous_skin_matrix = compute_previous_skin_matrix(entity, vertex, push_constants.bone_transform_buffer_device_address);
    vec4 previous_world_position = previous_skin_matrix * local_position;

    current_clip = scene_buffer.data.main_camera.view_projection * world_position;
    previous_clip = scene_buffer.data.main_camera.previous_view_projection * previous_world_position;

    gl_Position = scene_buffer.data.main_camera.jittered_view_projection * world_position;
}
