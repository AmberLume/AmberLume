#version 460

#include "../common.glsl"
#include "../skinning.glsl"
#include "push_constants.glsl"

layout(location = 0) out vec3 world_normal;

void main() {
    SceneBuffer scene_buffer = SceneBuffer(push_constants.scene_buffer_device_address);
    DrawData draw_data = DrawDataBuffer(push_constants.draw_data_buffer_device_address).data[gl_InstanceIndex];
    Entity entity = EntityBuffer(push_constants.entity_buffer_device_address).data[draw_data.entity_index];
    Vertex vertex = VertexBuffer(push_constants.vertex_buffer_device_address).data[gl_VertexIndex];

    mat4 skin_matrix = compute_skin_matrix(entity, vertex, push_constants.bone_transform_buffer_device_address);
    mat3 normal_matrix = mat3(transpose(inverse(skin_matrix)));
    vec4 world_position = skin_matrix * vec4(vertex.position[0], vertex.position[1], vertex.position[2], 1.0);

    world_normal = normalize(normal_matrix * vec3(vertex.normal[0], vertex.normal[1], vertex.normal[2]));

    gl_Position = scene_buffer.data.main_camera.view_projection * world_position;
}
