#ifndef PROJECTION_GLSL
#define PROJECTION_GLSL

vec4 jitter_clip_position(vec4 clip_position, vec2 jitter) {
    return vec4(clip_position.xy + jitter * clip_position.w, clip_position.zw);
}

vec3 world_position_from_depth(mat4 inverse_view_projection, vec2 jitter, ivec2 coord, ivec2 size, float depth) {
    vec2 uv = (vec2(coord) + 0.5) / vec2(size);
    vec2 ndc = uv * 2.0 - 1.0 - jitter;
    vec4 world_homogeneous = inverse_view_projection * vec4(ndc, depth, 1.0);

    return world_homogeneous.xyz / world_homogeneous.w;
}

#endif
