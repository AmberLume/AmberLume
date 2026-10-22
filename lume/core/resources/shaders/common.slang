struct Camera {
    float4x4 view_projection;
    float4x4 previous_view_projection;
    float4x4 inverse_view_projection;

    float4x4 view;

    float3 position;
    uint _pad0;

    float2 tan_half_fov;
    float near;
    float far;

    float2 jitter;
    float mip_bias;
    uint _pad1;
};

struct UiVertex {
    float2 position;
    float2 texture_coordinates;
    float4 color;
};
