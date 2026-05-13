#version 460

#extension GL_EXT_multiview : require

#include "../common.glsl"
#include "../skinning.glsl"
#include "push_constants.glsl"

void main() {
    DrawData draw_data = DrawDataBuffer(push_constants.draw_data_buffer_device_address).data[gl_InstanceIndex];

    if ((draw_data.cascade_mask & (1u << gl_ViewIndex)) == 0u) {
        gl_Position = vec4(2.0, 2.0, 2.0, 1.0);
        return;
    }

    Entity entity = EntityBuffer(push_constants.entity_buffer_device_address).data[draw_data.entity_index];
    Vertex vertex = VertexBuffer(push_constants.vertex_buffer_device_address).data[gl_VertexIndex];

    mat4 skin_matrix = compute_skin_matrix(entity, vertex, push_constants.bone_transform_buffer_device_address);
    vec4 world_position = skin_matrix * vec4(vertex.position[0], vertex.position[1], vertex.position[2], 1.0);

    ShadowCascadesBuffer cascades = ShadowCascadesBuffer(push_constants.shadow_cascades_buffer_device_address);
    gl_Position = cascades.data[gl_ViewIndex].light_space_matrix * world_position;
}
