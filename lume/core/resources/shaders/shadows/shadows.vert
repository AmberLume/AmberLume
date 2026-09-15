#version 460

#extension GL_EXT_multiview : require

#include "../common.glsl"
#include "../mesh_vertex.glsl"
#include "../shadow_cascade.glsl"
#include "push_constants.glsl"

void main() {
    DrawData draw_data = DrawDataBuffer(push_constants.draw_data_buffer_device_address).data[gl_InstanceIndex];

    if (!is_cascade_visible(draw_data.cascade_mask, gl_ViewIndex)) {
        gl_Position = CULLED_VERTEX_POSITION;
        return;
    }

    Entity entity = EntityBuffer(push_constants.entity_buffer_device_address).data[draw_data.entity_index];

    MeshVertex vertex = MeshVertexBuffer(entity.vertex_buffer_device_address).data[gl_VertexIndex];
    vec4 world_position = entity.transform_matrix * vec4(mesh_vertex_position(vertex), 1.0);

    gl_Position = cascade_clip_position(
        push_constants.shadow_cascades_buffer_device_address,
        gl_ViewIndex,
        world_position
    );
}
