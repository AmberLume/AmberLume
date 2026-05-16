#version 460

#include "push_constants.glsl"

layout(location = 0) in vec2 v_ndc;

layout(location = 0) out vec4 out_color;

vec3 hash33(vec3 p) {
    p = vec3(
        dot(p, vec3(127.1, 311.7, 74.7)),
        dot(p, vec3(269.5, 183.3, 246.1)),
        dot(p, vec3(113.5, 271.9, 124.6))
    );
    return fract(sin(p) * 43758.5453123);
}

vec3 view_direction() {
    vec4 near_point = push_constants.inverse_view_projection * vec4(v_ndc, 0.0, 1.0);
    vec4 far_point = push_constants.inverse_view_projection * vec4(v_ndc, 1.0, 1.0);

    return normalize(far_point.xyz / far_point.w - near_point.xyz / near_point.w);
}

vec3 star_layer(vec3 dir, float frequency, float probability, float core_size) {
    vec3 p = dir * frequency;
    vec3 id = floor(p);

    vec3 r1 = hash33(id);
    if (r1.x > probability) {
        return vec3(0.0);
    }

    vec3 r2 = hash33(id * 1.37 + 7.0);

    vec3 center = id + 0.5 + (r1.yzx - 0.5) * 0.7;
    float dist = length(p - center);

    float size = core_size * mix(0.8, 1.2, r2.x);
    float core = smoothstep(size, 0.0, dist);

    float phase = r1.y * 6.2831853;
    float speed = mix(1.5, 5.0, r1.z);
    float t = push_constants.time * speed + phase;
    float twinkle = 0.35 + 0.65 * (0.5 + 0.5 * sin(t));
    twinkle += 0.6 * pow(0.5 + 0.5 * sin(t * 1.7 + phase), 8.0);

    vec3 base = mix(push_constants.color_cool.rgb, push_constants.color_warm.rgb, r2.y);
    vec3 shimmer = mix(base, base.zyx, 0.12 + 0.12 * sin(t * 0.6 + r2.z * 6.2831853));
    vec3 tint = mix(vec3(1.0), shimmer, 0.55);

    return tint * core * twinkle;
}

void main() {
    vec3 dir = view_direction();

    vec3 sky = vec3(0.001, 0.001, 0.002);

    vec3 stars = star_layer(
        dir,
        push_constants.frequency,
        push_constants.probability,
        push_constants.core_size
    );

    out_color = vec4(sky + stars, 1.0);
}
