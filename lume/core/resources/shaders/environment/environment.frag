#version 460

#include "../sky.glsl"
#include "push_constants.glsl"

layout(location = 0) in vec2 v_ndc;

layout(location = 0) out vec4 out_color;

vec3 view_direction() {
    vec4 near_point = push_constants.inverse_view_projection * vec4(v_ndc, 0.0, 1.0);
    vec4 far_point = push_constants.inverse_view_projection * vec4(v_ndc, 1.0, 1.0);

    return normalize(far_point.xyz / far_point.w - near_point.xyz / near_point.w);
}

void main() {
    vec3 dir = view_direction();

    vec3 color = procedural_sky(dir, normalize(push_constants.sun_direction), push_constants.time, true);

    out_color = vec4(color, 1.0);
}
