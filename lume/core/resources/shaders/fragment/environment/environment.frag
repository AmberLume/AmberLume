#version 460

#include "../../common.glsl"
#include "../../sky.glsl"
#include "push_constants.glsl"

layout(location = 0) in vec2 in_uv;

layout(location = 0) out vec4 out_color;
layout(location = 1) out vec2 out_velocity;

vec3 view_direction(mat4 inverse_view_projection, vec2 ndc) {
    vec4 near_point = inverse_view_projection * vec4(ndc, 1.0, 1.0);
    vec4 far_point = inverse_view_projection * vec4(ndc, 0.0, 1.0);

    return normalize(far_point.xyz / far_point.w - near_point.xyz / near_point.w);
}

void main() {
    Scene scene = SceneBuffer(push_constants.fragment.scene_buffer_device_address).data;
    CameraBuffer camera = CameraBuffer(push_constants.fragment.camera_buffer_device_address);

    vec2 ndc = in_uv * 2.0 - 1.0;

    vec3 dir = view_direction(camera.inverse_view_projection, ndc);

    vec3 color = procedural_sky(dir, normalize(-scene.light_direction), scene.time, true);

    out_color = vec4(color, 1.0);

    vec4 previous_clip = camera.previous_view_projection * vec4(dir, 0.0);

    if (previous_clip.w > 0.0) {
        out_velocity = (previous_clip.xy / previous_clip.w - ndc) * 0.5;
    } else {
        out_velocity = vec2(2.0);
    }
}
