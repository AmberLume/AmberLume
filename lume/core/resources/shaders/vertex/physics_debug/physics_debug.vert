#version 460

#include "../../common.glsl"
#include "push_constants.glsl"

layout(push_constant, std430) uniform PushConstants {
    PhysicsDebugVertexShader vertex;
} push_constants;

layout(location = 0) out vec4 v_color;

void main() {
    CameraBuffer camera = CameraBuffer(push_constants.vertex.camera_buffer_device_address);

    PhysicsDebugVertex vertex = PhysicsDebugVertexBuffer(push_constants.vertex.physics_debug_vertex_buffer_device_address).data[gl_VertexIndex];

    gl_Position = camera.view_projection * vec4(vertex.point, 1.0);
    v_color = vertex.color;
}
