#version 460

#include "common.glsl"

layout(push_constant, std430) uniform PushConstants {
    uint64_t scene_buffer_device_address;
} push_constants;

layout(location = 0) out vec3 fragNormal;
layout(location = 1) out flat uint draw_id;

void main() {
    SceneGpuData scene = SceneBuffer(push_constants.scene_buffer_device_address).data;

    EntityBuffer entities = EntityBuffer(scene.entity_buffer_device_address);
    EntityGpuData entity = entities.data[gl_InstanceIndex];

    VertexBuffer vertices = VertexBuffer(scene.vertex_buffer_device_address);
    VertexGpuData vertex = vertices.data[gl_VertexIndex];

    vec4 world_position = entity.transform_matrix * vec4(vertex.position, 1.0);

    gl_Position = scene.projection_matrix * world_position;
    fragNormal = mat3(entity.transform_matrix) * vertex.normal;

    draw_id = gl_InstanceIndex;
}
