#version 460

#include "../common.glsl"
#include "../sky.glsl"
#include "push_constants.glsl"

layout(location = 0) in vec2 v_ndc;

layout(location = 0) out vec4 out_color;

vec3 view_direction(mat4 inverse_view_projection) {
    vec4 near_point = inverse_view_projection * vec4(v_ndc, 1.0, 1.0);
    vec4 far_point = inverse_view_projection * vec4(v_ndc, 0.0, 1.0);

    return normalize(far_point.xyz / far_point.w - near_point.xyz / near_point.w);
}

void main() {
    Scene scene = SceneBuffer(push_constants.scene_buffer_device_address).data;

    vec3 dir = view_direction(scene.main_camera.inverse_view_projection);

    vec3 color = procedural_sky(dir, normalize(-scene.light_direction), scene.time, true);

    out_color = vec4(color, 1.0);
}
