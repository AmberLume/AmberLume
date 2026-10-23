import "common.slang";

float4 jitter_clip_position(float4 clip_position, float2 jitter) {
    return float4(clip_position.xy + jitter * clip_position.w, clip_position.zw);
}

float3 world_position_from_depth(float4x4 inverse_view_projection, float2 jitter, int2 coord, int2 size, float depth) {
    float2 uv = (float2(coord) + 0.5) / float2(size);
    float2 ndc = uv * 2.0 - 1.0 - jitter;
    float4 world_homogeneous = mul(inverse_view_projection, float4(ndc, depth, 1.0));

    return world_homogeneous.xyz / world_homogeneous.w;
}

float3 view_ray_direction(float4x4 inverse_view_projection, float2 ndc) {
    float4 near_point = mul(inverse_view_projection, float4(ndc, 1.0, 1.0));
    float4 far_point = mul(inverse_view_projection, float4(ndc, 0.0, 1.0));

    return normalize(far_point.xyz / far_point.w - near_point.xyz / near_point.w);
}

float linear_view_z(Ptr<Camera, Access.Read> camera, float depth) {
    return (camera.near * camera.far) / (camera.near + depth * (camera.far - camera.near));
}

float pixel_footprint(Ptr<Camera, Access.Read> camera, float3 world_position, float height) {
    return 2.0 * camera.tan_half_fov.y * distance(world_position, camera.position) / height;
}

float2 screen_velocity(float4 current_clip, float4 previous_clip) {
    return (previous_clip.xy / previous_clip.w - current_clip.xy / current_clip.w) * 0.5;
}
