import "../bindings.slang";
import "../fullscreen.slang";
import "../texture.slang";

struct PushConstants {
    uint source_descriptor_id;
    uint karis_average;
    float threshold;
};

[[vk::push_constant]] ConstantBuffer<PushConstants> push_constants;

float karis_weight(float3 color) {
    float luma = dot(color, float3(0.2126, 0.7152, 0.0722));
    return 1.0 / (1.0 + luma);
}

float3 prefilter(float3 color, float threshold) {
    float knee = threshold * 0.5 + 0.0001;
    float brightness = max(color.r, max(color.g, color.b));
    float soft = clamp(brightness - threshold + knee, 0.0, 2.0 * knee);
    soft = soft * soft / (4.0 * knee);
    float contribution = max(soft, brightness - threshold) / max(brightness, 0.0001);
    return color * contribution;
}

[shader("vertex")]
FullscreenVertexOutput main(uint vertex_index : SV_VulkanVertexID) {
    return fullscreen_vertex(vertex_index);
}

[shader("fragment")]
float4 main(FullscreenVertexOutput input) : SV_Target0 {
    Texture2D source = graph_textures[push_constants.source_descriptor_id];
    SamplerState linear_clamp = samplers[SAMPLER_LINEAR_CLAMP];
    float2 texel = 1.0 / float2(texture_size(source));

    float3 a = source.Sample(linear_clamp, input.uv + texel * float2(-2.0, 2.0)).rgb;
    float3 b = source.Sample(linear_clamp, input.uv + texel * float2(0.0, 2.0)).rgb;
    float3 c = source.Sample(linear_clamp, input.uv + texel * float2(2.0, 2.0)).rgb;
    float3 d = source.Sample(linear_clamp, input.uv + texel * float2(-2.0, 0.0)).rgb;
    float3 e = source.Sample(linear_clamp, input.uv).rgb;
    float3 f = source.Sample(linear_clamp, input.uv + texel * float2(2.0, 0.0)).rgb;
    float3 g = source.Sample(linear_clamp, input.uv + texel * float2(-2.0, -2.0)).rgb;
    float3 h = source.Sample(linear_clamp, input.uv + texel * float2(0.0, -2.0)).rgb;
    float3 i = source.Sample(linear_clamp, input.uv + texel * float2(2.0, -2.0)).rgb;
    float3 j = source.Sample(linear_clamp, input.uv + texel * float2(-1.0, 1.0)).rgb;
    float3 k = source.Sample(linear_clamp, input.uv + texel * float2(1.0, 1.0)).rgb;
    float3 l = source.Sample(linear_clamp, input.uv + texel * float2(-1.0, -1.0)).rgb;
    float3 m = source.Sample(linear_clamp, input.uv + texel * float2(1.0, -1.0)).rgb;

    float3 result;
    if (push_constants.karis_average == 1u) {
        float threshold = push_constants.threshold;
        a = prefilter(a, threshold);
        b = prefilter(b, threshold);
        c = prefilter(c, threshold);
        d = prefilter(d, threshold);
        e = prefilter(e, threshold);
        f = prefilter(f, threshold);
        g = prefilter(g, threshold);
        h = prefilter(h, threshold);
        i = prefilter(i, threshold);
        j = prefilter(j, threshold);
        k = prefilter(k, threshold);
        l = prefilter(l, threshold);
        m = prefilter(m, threshold);

        float3 group_center = (j + k + l + m) * 0.25;
        float3 group_top_left = (a + b + d + e) * 0.25;
        float3 group_top_right = (b + c + e + f) * 0.25;
        float3 group_bottom_left = (d + e + g + h) * 0.25;
        float3 group_bottom_right = (e + f + h + i) * 0.25;

        float weight_center = karis_weight(group_center) * 0.5;
        float weight_top_left = karis_weight(group_top_left) * 0.125;
        float weight_top_right = karis_weight(group_top_right) * 0.125;
        float weight_bottom_left = karis_weight(group_bottom_left) * 0.125;
        float weight_bottom_right = karis_weight(group_bottom_right) * 0.125;

        result = group_center * weight_center
            + group_top_left * weight_top_left
            + group_top_right * weight_top_right
            + group_bottom_left * weight_bottom_left
            + group_bottom_right * weight_bottom_right;
        result /= weight_center + weight_top_left + weight_top_right + weight_bottom_left + weight_bottom_right;
    } else {
        result = e * 0.125;
        result += (a + c + g + i) * 0.03125;
        result += (b + d + f + h) * 0.0625;
        result += (j + k + l + m) * 0.125;
    }

    return float4(result, 1.0);
}
