#version 460

#include "../common.glsl"
#include "push_constants.glsl"

layout(location = 0) in vec4 v_color;

layout(location = 0) out vec4 out_color;

void main() {
    out_color = v_color;
}
