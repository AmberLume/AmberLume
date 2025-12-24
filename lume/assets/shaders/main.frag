#version 460
#extension GL_EXT_buffer_reference : require

layout(location = 0) out vec4 out_color;

void main() {
    out_color = vec4(1.0, 0.2, 0.2, 1.0);
}
