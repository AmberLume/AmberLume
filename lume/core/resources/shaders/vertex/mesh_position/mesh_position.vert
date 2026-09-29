#version 460

#extension GL_EXT_multiview : require

#include "../../common.glsl"
#include "../../mesh_vertex.glsl"
#include "../../shadow_cascade.glsl"
#include "push_constants.glsl"

layout(push_constant, std430) uniform PushConstants {
    MeshPositionVertexShader vertex;
} push_constants;

void main() {
    DrawData draw_data = DrawDataBuffer(push_constants.vertex.draw_data_buffer_device_address).data[gl_InstanceIndex];

    if (!is_cascade_visible(draw_data.cascade_mask, gl_ViewIndex)) {
        gl_Position = CULLED_VERTEX_POSITION;
        return;
    }

    Entity entity = EntityBuffer(push_constants.vertex.entity_buffer_device_address).data[draw_data.entity_index];

    VertexPositionBuffer positions = VertexPositionBuffer(push_constants.vertex.vertex_position_buffer_device_address);

    vec4 world_position = entity.transform_matrix * vec4(vertex_position(positions, uint(gl_VertexIndex)), 1.0);

    gl_Position = cascade_clip_position(
        push_constants.vertex.shadow_cascades_buffer_device_address,
        gl_ViewIndex,
        world_position
    );
}
