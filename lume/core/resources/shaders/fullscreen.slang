struct FullscreenVertexOutput {
    float4 position : SV_Position;
    float2 uv : UV;
};

float2 fullscreen_uv(uint vertex_index) {
    return float2(float((vertex_index << 1) & 2), float(vertex_index & 2));
}

FullscreenVertexOutput fullscreen_vertex(uint vertex_index) {
    float2 uv = fullscreen_uv(vertex_index);

    FullscreenVertexOutput output;
    output.position = float4(uv * 2.0 - 1.0, 0.0, 1.0);
    output.uv = uv;

    return output;
}

float4 fullscreen_position(uint vertex_index) {
    return float4(fullscreen_uv(vertex_index) * 2.0 - 1.0, 1.0, 1.0);
}
