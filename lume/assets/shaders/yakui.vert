#version 460

#include "common.glsl"

layout(location = 0) out vec4 out_color;
layout(location = 1) out vec2 out_texcoord;

layout(push_constant, std430) uniform PushConstants {
    uint64_t scene_buffer_device_address;
} push_constants;

void main() {
    SceneGpuData scene = SceneBuffer(push_constants.scene_buffer_device_address).data;

    UiVertexBuffer ui_vertices = UiVertexBuffer(scene.ui_vertex_buffer_device_address);
    UiVertexGpuData ui_vertex = ui_vertices.data[gl_VertexIndex];

    vec2 position = ui_vertex.position * 2.0 - 1.0;

    gl_Position = vec4(position, 0.0, 1.0);

    out_color = vec4(1.0, 0.0, 0.0, 1.0);
    out_color = ui_vertex.color;
    out_texcoord = ui_vertex.texcoord;
}
