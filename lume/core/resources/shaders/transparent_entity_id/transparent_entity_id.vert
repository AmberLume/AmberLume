#version 460

#include "../common.glsl"
#include "../projection.glsl"
#include "../mesh_vertex.glsl"
#include "push_constants.glsl"

layout(location = 0) out flat uint entity_index;
layout(location = 1) out vec4 current_clip;
layout(location = 2) out vec4 previous_clip;

void main() {
    CameraBuffer camera = CameraBuffer(push_constants.camera_buffer_device_address);
    DrawData draw_data = DrawDataBuffer(push_constants.draw_data_buffer_device_address).data[gl_InstanceIndex];
    Entity entity = EntityBuffer(push_constants.entity_buffer_device_address).data[draw_data.entity_index];
    EntityMotion entity_motion = EntityMotionBuffer(push_constants.entity_motion_buffer_device_address).data[draw_data.entity_index];

    MeshVertex vertex = MeshVertexBuffer(entity.vertex_buffer_device_address).data[gl_VertexIndex];
    MeshVertex previous_vertex = MeshVertexBuffer(entity_motion.previous_vertex_buffer_device_address).data[gl_VertexIndex];

    entity_index = draw_data.entity_index;

    vec4 local_position = vec4(mesh_vertex_position(vertex), 1.0);

    vec4 world_position = entity.transform_matrix * local_position;

    vec4 previous_world_position = entity_motion.previous_transform_matrix * vec4(mesh_vertex_position(previous_vertex), 1.0);

    current_clip = camera.view_projection * world_position;
    previous_clip = camera.previous_view_projection * previous_world_position;

    gl_Position = jitter_clip_position(current_clip, camera.jitter);
}
