#version 460

#include "../../common.glsl"
#include "../../projection.glsl"
#include "../../mesh_vertex.glsl"
#include "push_constants.glsl"

layout(push_constant, std430) uniform PushConstants {
    MeshNormalVelocityVertexShader vertex;
} push_constants;

layout(location = 0) out vec3 world_normal;
layout(location = 1) out vec4 current_clip;
layout(location = 2) out vec4 previous_clip;

void main() {
    CameraBuffer camera = CameraBuffer(push_constants.vertex.camera_buffer_device_address);
    DrawData draw_data = DrawDataBuffer(push_constants.vertex.draw_data_buffer_device_address).data[gl_InstanceIndex];
    Entity entity = EntityBuffer(push_constants.vertex.entity_buffer_device_address).data[draw_data.entity_index];
    Submesh submesh = SubmeshBuffer(push_constants.vertex.submesh_buffer_device_address).data[draw_data.submesh_index];

    VertexPositionBuffer positions = VertexPositionBuffer(push_constants.vertex.vertex_position_buffer_device_address);
    VertexNormalTangentBuffer normal_tangents = VertexNormalTangentBuffer(push_constants.vertex.vertex_normal_tangent_buffer_device_address);

    uint vertex_index = uint(gl_VertexIndex);
    uint previous_vertex_index = submesh.previous_vertex_offset + (vertex_index - submesh.vertex_offset);

    vec4 local_position = vec4(vertex_position(positions, vertex_index), 1.0);

    mat3 normal_matrix = mat3(transpose(inverse(entity.transform_matrix)));
    vec4 world_position = entity.transform_matrix * local_position;

    vec4 previous_world_position = entity.previous_transform_matrix * vec4(vertex_position(positions, previous_vertex_index), 1.0);

    world_normal = normalize(normal_matrix * vertex_normal(normal_tangents, vertex_index));

    current_clip = camera.view_projection * world_position;
    previous_clip = camera.previous_view_projection * previous_world_position;

    gl_Position = jitter_clip_position(current_clip, camera.jitter);
}
