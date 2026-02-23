#version 460

#extension GL_ARB_shader_draw_parameters : enable

#include "../common.glsl"
#include "push_constants.glsl"

layout(location = 0) out vec3 frag_normal;
layout(location = 1) out vec2 uv;
layout(location = 2) out flat uint draw_id;

void main() {
    DrawDataGpuData draw_data = DrawDataBuffer(push_constants.draw_data_buffer_device_address).data[gl_DrawIDARB];
    EntityGpuData entity = EntityBuffer(push_constants.entity_buffer_device_address).data[draw_data.entity_index];
    VertexGpuData vertex = VertexBuffer(push_constants.vertex_buffer_device_address).data[gl_VertexIndex];

    vec4 world_position = entity.transform_matrix * vec4(vertex.position[0], vertex.position[1], vertex.position[2], 1.0);

    gl_Position = push_constants.projection_matrix * world_position;

    frag_normal = mat3(entity.transform_matrix) * vec3(vertex.normal[0], vertex.normal[1], vertex.normal[2]);
    uv = vec2(vertex.uv[0], vertex.uv[1]);
    draw_id = gl_DrawIDARB;
}
