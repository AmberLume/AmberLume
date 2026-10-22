float3 world_position_from_depth(float4x4 inverse_view_projection, float2 jitter, int2 coord, int2 size, float depth) {
    float2 uv = (float2(coord) + 0.5) / float2(size);
    float2 ndc = uv * 2.0 - 1.0 - jitter;
    float4 world_homogeneous = mul(inverse_view_projection, float4(ndc, depth, 1.0));

    return world_homogeneous.xyz / world_homogeneous.w;
}
