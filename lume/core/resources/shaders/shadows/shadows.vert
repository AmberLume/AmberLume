#version 460

#extension GL_EXT_multiview : require

#include "../common.glsl"
#include "../mesh_vertex.glsl"
#include "../shadow_cascade.glsl"
#include "../skinning.glsl"
#include "push_constants.glsl"

void main() {
    DrawData draw_data = DrawDataBuffer(push_constants.draw_data_buffer_device_address).data[gl_InstanceIndex];

    if (!is_cascade_visible(draw_data.cascade_mask, gl_ViewIndex)) {
        gl_Position = CULLED_VERTEX_POSITION;
        return;
    }

    Entity entity = EntityBuffer(push_constants.entity_buffer_device_address).data[draw_data.entity_index];
    Submesh submesh = SubmeshBuffer(push_constants.submesh_buffer_device_address).data[draw_data.submesh_index];

    MeshVertex vertex = MeshVertexBuffer(push_constants.mesh_vertex_buffer_device_address).data[gl_VertexIndex];

    mat4 skin_matrix = compute_skin_matrix(
        entity.transform_matrix,
        entity.bone_transform_offset,
        submesh.vertex_skin_offset + uint(gl_VertexIndex) - submesh.vertex_offset,
        push_constants.mesh_vertex_skin_buffer_device_address,
        push_constants.bone_transform_buffer_device_address
    );
    vec4 world_position = skin_matrix * vec4(mesh_vertex_position(vertex), 1.0);

    gl_Position = cascade_clip_position(
        push_constants.shadow_cascades_buffer_device_address,
        gl_ViewIndex,
        world_position
    );
}
