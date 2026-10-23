import "../bindings.slang";
import "../fullscreen.slang";
import "../texture.slang";

struct PushConstants {
    uint source_descriptor_id;
};

[[vk::push_constant]] ConstantBuffer<PushConstants> push_constants;

[shader("vertex")]
FullscreenVertexOutput main(uint vertex_index : SV_VulkanVertexID) {
    return fullscreen_vertex(vertex_index);
}

[shader("fragment")]
float4 main(FullscreenVertexOutput input) : SV_Target0 {
    Texture2D source = graph_textures[push_constants.source_descriptor_id];
    SamplerState linear_clamp = samplers[SAMPLER_LINEAR_CLAMP];
    float2 texel = 1.0 / float2(texture_size(source));

    float3 result = source.Sample(linear_clamp, input.uv).rgb * 4.0;
    result += source.Sample(linear_clamp, input.uv + texel * float2(1.0, 0.0)).rgb * 2.0;
    result += source.Sample(linear_clamp, input.uv + texel * float2(-1.0, 0.0)).rgb * 2.0;
    result += source.Sample(linear_clamp, input.uv + texel * float2(0.0, 1.0)).rgb * 2.0;
    result += source.Sample(linear_clamp, input.uv + texel * float2(0.0, -1.0)).rgb * 2.0;
    result += source.Sample(linear_clamp, input.uv + texel * float2(1.0, 1.0)).rgb;
    result += source.Sample(linear_clamp, input.uv + texel * float2(-1.0, 1.0)).rgb;
    result += source.Sample(linear_clamp, input.uv + texel * float2(1.0, -1.0)).rgb;
    result += source.Sample(linear_clamp, input.uv + texel * float2(-1.0, -1.0)).rgb;
    result *= 1.0 / 16.0;

    return float4(result, 1.0);
}
