#version 460

#extension GL_ARB_shader_draw_parameters : enable

#include "common.glsl"

layout(push_constant, std430) uniform PushConstants {
    uint64_t scene_buffer_device_address;
} push_constants;

layout(location = 0) out vec3 frag_normal;
layout(location = 1) out vec2 uv;
layout(location = 2) out flat uint draw_id;

void main() {
    SceneGpuData scene = SceneBuffer(push_constants.scene_buffer_device_address).data;

    DrawBufferRead draws = DrawBufferRead(scene.draw_buffer_device_address);
    DrawGpuData draw = draws.data[gl_DrawIDARB];

    EntityBuffer entities = EntityBuffer(scene.entity_buffer_device_address);
    EntityGpuData entity = entities.data[draw.entity_index];

    VertexBuffer vertices = VertexBuffer(scene.vertex_buffer_device_address);
    VertexGpuData vertex = vertices.data[gl_VertexIndex];

    vec4 world_position = entity.transform_matrix * vec4(vertex.position[0], vertex.position[1], vertex.position[2], 1.0);

    gl_Position = scene.projection_matrix * world_position;

    frag_normal = mat3(entity.transform_matrix) * vec3(vertex.normal[0], vertex.normal[1], vertex.normal[2]);
    uv = vec2(vertex.uv[0], vertex.uv[1]);
    draw_id = gl_DrawIDARB;
}
