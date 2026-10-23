import "../bindings.slang";

struct PushConstants {
    uint current_guide_descriptor_id;
    uint previous_guide_descriptor_id;
    uint velocity_descriptor_id;
    uint noisy_descriptor_id;
    uint previous_signal_descriptor_id;
    uint signal_storage_id;

    uint width;
    uint height;
    uint history_valid;
    uint history_frames;
    uint frame_number;
    uint colored;

    float depth_threshold;
    float normal_threshold;
};

[[vk::push_constant]] ConstantBuffer<PushConstants> push_constants;

float3 signal(float4 value, bool colored) {
    return colored ? value.rgb : value.rrr;
}

[shader("compute")]
[numthreads(16, 16, 1)]
void main(uint3 global_invocation_id : SV_DispatchThreadID) {
    int2 coord = int2(global_invocation_id.xy);
    int2 size = int2(push_constants.width, push_constants.height);
    if (any(coord >= size)) {
        return;
    }

    SamplerState linear_clamp = samplers[SAMPLER_LINEAR_CLAMP];
    Texture2D noisy_texture = graph_textures[push_constants.noisy_descriptor_id];

    float4 guide_current = graph_textures[push_constants.current_guide_descriptor_id].Load(int3(coord, 0));
    float linear_z = guide_current.r;
    float3 normal = guide_current.gba;

    bool reprojected = false;
    float2 history_uv = float2(0.0);

    if (push_constants.history_valid != 0u && linear_z > 0.0) {
        float2 uv = (float2(coord) + 0.5) / float2(size);
        float2 motion = graph_textures[push_constants.velocity_descriptor_id].Load(int3(coord, 0)).rg;
        history_uv = uv + motion;

        bool on_screen = all(history_uv >= 0.0) && all(history_uv <= 1.0);

        if (on_screen) {
            float4 guide_previous = graph_textures[push_constants.previous_guide_descriptor_id].SampleLevel(linear_clamp, history_uv, 0.0);

            reprojected = guide_previous.r > 0.0
                && abs(linear_z - guide_previous.r) < push_constants.depth_threshold * linear_z
                && dot(normal, guide_previous.gba) > push_constants.normal_threshold;
        }
    }

    bool colored = push_constants.colored != 0u;

    float3 noisy = signal(noisy_texture.Load(int3(coord, 0)), colored);

    float3 integrated = noisy;
    float history_length = 1.0;

    if (reprojected) {
        float4 history = graph_textures[push_constants.previous_signal_descriptor_id].SampleLevel(linear_clamp, history_uv, 0.0);

        float3 history_signal = signal(history, colored);
        float history_length_previous = colored ? history.a : history.g;

        history_length = min(history_length_previous + 1.0, float(push_constants.history_frames));
        float alpha = max(1.0 / float(push_constants.history_frames), 1.0 / history_length);

        float3 sum = float3(0.0);
        float3 sum_squared = float3(0.0);
        for (int y = -1; y <= 1; y++) {
            for (int x = -1; x <= 1; x++) {
                int2 tap = clamp(coord + int2(x, y), int2(0), size - 1);
                float3 value = signal(noisy_texture.Load(int3(tap, 0)), colored);
                sum += value;
                sum_squared += value * value;
            }
        }

        float3 mean = sum / 9.0;
        float3 sigma = sqrt(max(sum_squared / 9.0 - mean * mean, float3(0.0)));
        float3 reference = clamp(history_signal, mean - sigma, mean + sigma);

        integrated = lerp(reference, noisy, alpha);
    }

    storage_images_rgba16f[push_constants.signal_storage_id][coord] = colored
        ? float4(integrated, history_length)
        : float4(integrated.r, history_length, 0.0, 0.0);
}
