#version 460

#include "../common.glsl"
#include "../sh.glsl"
#include "../sky.glsl"
#include "sh_project_push_constants.glsl"

layout(location = 0) out vec4 out_color;

const uint SAMPLE_COUNT = 64u;
const float PI = 3.14159265359;

vec3 fibonacci_direction(uint i, uint n) {
    float k = float(i) + 0.5;
    float cos_phi = 1.0 - 2.0 * k / float(n);
    float sin_phi = sqrt(max(1.0 - cos_phi * cos_phi, 0.0));
    float theta = PI * (1.0 + sqrt(5.0)) * k;

    return vec3(sin_phi * cos(theta), sin_phi * sin(theta), cos_phi);
}

void main() {
    uint coeff = uint(gl_FragCoord.x);
    Scene scene = SceneBuffer(push_constants.scene_buffer_device_address).data;
    vec3 sun = normalize(-scene.light_direction);

    float dw = 4.0 * PI / float(SAMPLE_COUNT);

    vec3 acc = vec3(0.0);
    for (uint s = 0u; s < SAMPLE_COUNT; s++) {
        vec3 dir = fibonacci_direction(s, SAMPLE_COUNT);
        vec3 radiance = procedural_sky(dir, sun, scene.time, false);

        float y[9];
        sh_basis(dir, y);

        acc += radiance * y[coeff] * dw;
    }

    out_color = vec4(acc, 1.0);
}
