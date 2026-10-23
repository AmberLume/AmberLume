import "../bindings.slang";
import "../fullscreen.slang";
import "../texture.slang";

struct PushConstants {
    uint input_descriptor_id;
    float exposure;
    float saturation;
    float contrast;
    float offset[3];
    float gamma[3];
    float gain[3];
    uint hdr;
    float paper_white;
    float display_peak;
    uint bloom_descriptor_id;
    float bloom_intensity;
    float sharpness;
};

[[vk::push_constant]] ConstantBuffer<PushConstants> push_constants;

static const float3 REC709_LUMINANCE = float3(0.2126, 0.7152, 0.0722);
static const float MIDDLE_GREY = 0.18;

static const float PBR_NEUTRAL_F90 = 0.04;
static const float PBR_NEUTRAL_COMPRESSION_START = 0.8;
static const float PBR_NEUTRAL_DESATURATION = 0.15;

float3 color_grade(float3 color, float saturation, float contrast, float3 offset, float3 gamma, float3 gain) {
    float luminance = dot(color, REC709_LUMINANCE);
    color = max(lerp(float3(luminance), color, saturation), 0.0);
    color = MIDDLE_GREY * pow(color / MIDDLE_GREY, float3(contrast));
    color = pow(color, 1.0 / gamma);

    return max(color * gain + offset, 0.0);
}

float3 pbr_neutral(float3 color, float display_peak) {
    float lowest = min(color.r, min(color.g, color.b));
    float offset = lowest < 2.0 * PBR_NEUTRAL_F90 ? lowest - lowest * lowest / (4.0 * PBR_NEUTRAL_F90) : PBR_NEUTRAL_F90;
    color -= offset;

    float peak = max(color.r, max(color.g, color.b));
    float start_compression = PBR_NEUTRAL_COMPRESSION_START * display_peak - PBR_NEUTRAL_F90;
    if (peak < start_compression) {
        return color;
    }

    float shoulder = display_peak - start_compression;
    float compressed_peak = display_peak - shoulder * shoulder / (peak + shoulder - start_compression);
    color *= compressed_peak / peak;

    float desaturation = 1.0 - 1.0 / (PBR_NEUTRAL_DESATURATION * (peak - compressed_peak) + 1.0);
    return lerp(color, float3(compressed_peak), desaturation);
}

float3 sharpen(Texture2D source, SamplerState linear_clamp, float2 uv) {
    float3 center = source.Sample(linear_clamp, uv).rgb;

    if (push_constants.sharpness <= 0.0) {
        return center;
    }

    float2 texel = 1.0 / float2(texture_size(source));

    float3 up = source.Sample(linear_clamp, uv + float2(0.0, -texel.y)).rgb;
    float3 down = source.Sample(linear_clamp, uv + float2(0.0, texel.y)).rgb;
    float3 left = source.Sample(linear_clamp, uv + float2(-texel.x, 0.0)).rgb;
    float3 right = source.Sample(linear_clamp, uv + float2(texel.x, 0.0)).rgb;

    float3 lowest = min(center, min(min(up, down), min(left, right)));
    float3 highest = max(center, max(max(up, down), max(left, right)));

    float3 sharpened = center + (center * 4.0 - (up + down + left + right)) * (push_constants.sharpness * 0.4);

    return clamp(sharpened, lowest, highest);
}

[shader("vertex")]
FullscreenVertexOutput main(uint vertex_index : SV_VulkanVertexID) {
    return fullscreen_vertex(vertex_index);
}

[shader("fragment")]
float4 main(FullscreenVertexOutput input) : SV_Target0 {
    SamplerState linear_clamp = samplers[SAMPLER_LINEAR_CLAMP];

    float3 color = sharpen(graph_textures[push_constants.input_descriptor_id], linear_clamp, input.uv);

    if (push_constants.bloom_intensity > 0.0) {
        color += graph_textures[push_constants.bloom_descriptor_id].Sample(linear_clamp, input.uv).rgb * push_constants.bloom_intensity;
    }

    color *= push_constants.exposure;

    float3 offset = float3(push_constants.offset[0], push_constants.offset[1], push_constants.offset[2]);
    float3 gamma = float3(push_constants.gamma[0], push_constants.gamma[1], push_constants.gamma[2]);
    float3 gain = float3(push_constants.gain[0], push_constants.gain[1], push_constants.gain[2]);
    color = color_grade(color, push_constants.saturation, push_constants.contrast, offset, gamma, gain);
    color = pbr_neutral(color, push_constants.display_peak);

    if (push_constants.hdr == 1u) {
        color *= push_constants.paper_white;
    }

    return float4(color, 1.0);
}
