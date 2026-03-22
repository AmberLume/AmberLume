#version 460

#include "../common.glsl"
#include "push_constants.glsl"

void main() {
    SceneBuffer scene_buffer = SceneBuffer(push_constants.scene_buffer_device_address);
    DrawDataGpuData draw_data = DrawDataBuffer(push_constants.draw_data_buffer_device_address).data[gl_InstanceIndex];
    EntityGpuData entity = EntityBuffer(push_constants.entity_buffer_device_address).data[draw_data.entity_index];
    VertexGpuData vertex = VertexBuffer(push_constants.vertex_buffer_device_address).data[gl_VertexIndex];

    vec4 world_position = entity.transform_matrix * vec4(vertex.position[0], vertex.position[1], vertex.position[2], 1.0);

    gl_Position = scene_buffer.data.main_camera.projection_matrix * world_position;
}
