#version 460

#include "../common.glsl"
#include "push_constants.glsl"

void main() {
    SceneBuffer scene_buffer = SceneBuffer(push_constants.scene_buffer_device_address);
    EntityGpuData entity = EntityBuffer(push_constants.entity_buffer_device_address).data[gl_InstanceIndex];
    VertexGpuData vertex = VertexBuffer(push_constants.vertex_buffer_device_address).data[gl_VertexIndex];

    vec4 world_position = entity.transform_matrix * vec4(vertex.position[0], vertex.position[1], vertex.position[2], 1.0);

    gl_Position = scene_buffer.data.shadow_cascades[push_constants.shadow_cascade_index].light_space_matrix * world_position;
}
