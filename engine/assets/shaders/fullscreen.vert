#version 450

vec2 FULLSCREEN_TRIANGLE[3] = vec2[](
    vec2(-1, -1),
    vec2(3, -1),
    vec2(-1, 3)
);

void main() {
    vec2 position = FULLSCREEN_TRIANGLE[gl_VertexIndex];
    gl_Position = vec4(position, 0.0, 1.0);
}
