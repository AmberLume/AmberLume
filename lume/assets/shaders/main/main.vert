#version 460

#extension GL_ARB_shader_draw_parameters : enable

#include "../common.glsl"
#include "push_constants.glsl"

layout(location = 0) out mat3 out_TBN;
layout(location = 3) out vec2 uv;
layout(location = 4) out flat uint draw_id;
layout(location = 5) out vec3 world_pos;

void main() {
    SceneBuffer scene_buffer = SceneBuffer(push_constants.scene_buffer_device_address);
    DrawDataGpuData draw_data = DrawDataBuffer(push_constants.draw_data_buffer_device_address).data[gl_DrawIDARB];
    EntityGpuData entity = EntityBuffer(push_constants.entity_buffer_device_address).data[draw_data.entity_index];
    VertexGpuData vertex = VertexBuffer(push_constants.vertex_buffer_device_address).data[gl_VertexIndex];

    vec4 world_position = entity.transform_matrix * vec4(vertex.position[0], vertex.position[1], vertex.position[2], 1.0);
    gl_Position = scene_buffer.data.main_camera.projection_matrix * world_position;

    mat3 model_mat = mat3(entity.transform_matrix);

    vec3 T = normalize(model_mat * vec3(vertex.tangent[0], vertex.tangent[1], vertex.tangent[2]));
    vec3 N = normalize(model_mat * vec3(vertex.normal[0], vertex.normal[1], vertex.normal[2]));

    T = normalize(T - dot(T, N) * N);

    vec3 B = cross(N, T) * vertex.tangent[3];

    out_TBN = mat3(T, B, N);
    uv = vec2(vertex.uv[0], vertex.uv[1]);
    draw_id = gl_DrawIDARB;
    world_pos = world_position.xyz;
}
