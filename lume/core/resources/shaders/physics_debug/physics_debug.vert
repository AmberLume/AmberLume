#version 460

#include "../common.glsl"
#include "push_constants.glsl"

layout(location = 0) out vec4 v_color;

void main() {
    Scene scene = SceneBuffer(push_constants.scene_buffer_device_address).data;

    PhysicsDebugVertex vertex = PhysicsDebugVertexBuffer(push_constants.physics_debug_vertices_buffer_device_address).data[gl_VertexIndex];

    gl_Position = scene.main_camera.view_projection * vec4(vertex.point, 1.0);
    v_color = vertex.color;
}
