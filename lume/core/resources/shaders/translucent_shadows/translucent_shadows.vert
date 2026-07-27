#version 460

#extension GL_EXT_multiview : require

#include "../common.glsl"
#include "../shadow_cascade.glsl"
#include "../skinning.glsl"
#include "push_constants.glsl"

layout(location = 0) out vec2 uv;
layout(location = 1) out flat uint draw_id;

void main() {
    draw_id = gl_InstanceIndex;

    DrawData draw_data = DrawDataBuffer(push_constants.draw_data_buffer_device_address).data[gl_InstanceIndex];

    Vertex vertex = VertexBuffer(push_constants.vertex_buffer_device_address).data[gl_VertexIndex];
    uv = vec2(vertex.uv[0], vertex.uv[1]);

    if (!is_cascade_visible(draw_data.cascade_mask, gl_ViewIndex)) {
        gl_Position = CULLED_VERTEX_POSITION;
        return;
    }

    Entity entity = EntityBuffer(push_constants.entity_buffer_device_address).data[draw_data.entity_index];

    mat4 skin_matrix = compute_skin_matrix(entity, vertex, push_constants.bone_transform_buffer_device_address);
    vec4 world_position = skin_matrix * vec4(vertex.position[0], vertex.position[1], vertex.position[2], 1.0);

    gl_Position = cascade_clip_position(
        push_constants.shadow_cascades_buffer_device_address,
        gl_ViewIndex,
        world_position
    );
}
