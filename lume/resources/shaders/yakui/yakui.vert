#version 460

#include "../common.glsl"
#include "push_constants.glsl"

layout(location = 0) out vec4 out_color;
layout(location = 1) out vec2 out_texcoord;

void main() {
    UiVertexGpuData ui_vertex = UiVertexBuffer(push_constants.ui_vertex_buffer_device_address).data[gl_VertexIndex];

    vec2 position = ui_vertex.position * 2.0 - 1.0;

    gl_Position = vec4(position, 0.0, 1.0);

    out_color = vec4(1.0, 0.0, 0.0, 1.0);
    out_color = ui_vertex.color;
    out_texcoord = ui_vertex.texcoord;
}
