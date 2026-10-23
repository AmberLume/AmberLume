void transform_aabb(float4x4 transform, float3 local_min, float3 local_max, out float3 transformed_min, out float3 transformed_max) {
    float3 local_center = (local_min + local_max) * 0.5;
    float3 local_extents = (local_max - local_min) * 0.5;

    float3 center = mul(transform, float4(local_center, 1.0)).xyz;
    float3 extents = mul(abs((float3x3)transform), local_extents);

    transformed_min = center - extents;
    transformed_max = center + extents;
}
