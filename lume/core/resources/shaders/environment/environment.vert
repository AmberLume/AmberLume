#version 460

#include "push_constants.glsl"

layout(location = 0) out vec2 v_ndc;

void main() {
    vec2 ndc = vec2(
        float((gl_VertexIndex << 1) & 2),
        float(gl_VertexIndex & 2)
    ) * 2.0 - 1.0;

    gl_Position = vec4(ndc, 0.0, 1.0);

    v_ndc = ndc;
}
