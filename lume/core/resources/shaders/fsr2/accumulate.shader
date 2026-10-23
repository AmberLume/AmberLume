import "../bindings.slang";
import "../texture.slang";

struct PushConstants {
    uint scene_color_descriptor_id;
    uint velocity_descriptor_id;
    uint previous_history_descriptor_id;
    uint current_history_storage_id;
    uint history_valid;
    uint display_width;
    uint display_height;
};

[[vk::push_constant]] ConstantBuffer<PushConstants> push_constants;

static const float MOTION_EPSILON_PIXELS = 0.1;
static const float MOTION_BLEND_PIXELS = 32.0;
static const float ACCUMULATION_FRAMES_STILL = 10.0;
static const float ACCUMULATION_FRAMES_MOVING = 1.2;

float3 rgb_to_ycocg(float3 c) {
    return float3(
        0.25 * c.r + 0.5 * c.g + 0.25 * c.b,
        0.5 * c.r - 0.5 * c.b,
        -0.25 * c.r + 0.5 * c.g - 0.25 * c.b
    );
}

float3 ycocg_to_rgb(float3 c) {
    return float3(
        c.x + c.y - c.z,
        c.x + c.z,
        c.x - c.y - c.z
    );
}

float3 tonemap(float3 color) {
    return color / (1.0 + max(max(color.r, color.g), color.b));
}

float3 inverse_tonemap(float3 color) {
    return color / max(1.0 - max(max(color.r, color.g), color.b), 1e-3);
}

[shader("compute")]
[numthreads(16, 16, 1)]
void main(uint3 global_invocation_id : SV_DispatchThreadID) {
    int2 coord = int2(global_invocation_id.xy);
    int2 size = int2(push_constants.display_width, push_constants.display_height);
    if (any(coord >= size)) {
        return;
    }

    SamplerState linear_clamp = samplers[SAMPLER_LINEAR_CLAMP];
    Texture2D scene_texture = graph_textures[push_constants.scene_color_descriptor_id];
    RWTexture2D<float4> history_current = storage_images_rgba16f[push_constants.current_history_storage_id];

    float2 display_size = float2(size);
    float2 uv = (float2(coord) + 0.5) / display_size;

    float3 scene = scene_texture.SampleLevel(linear_clamp, uv, 0.0).rgb;

    if (push_constants.history_valid == 0u) {
        history_current[coord] = float4(scene, 1.0);
        return;
    }

    float2 motion = graph_textures[push_constants.velocity_descriptor_id].SampleLevel(linear_clamp, uv, 0.0).rg;

    float motion_pixels = length(motion * display_size);

    if (motion_pixels < MOTION_EPSILON_PIXELS) {
        motion = float2(0.0);
        motion_pixels = 0.0;
    }

    float2 history_uv = uv + motion;

    bool on_screen = all(history_uv >= 0.0) && all(history_uv <= 1.0);
    if (!on_screen) {
        history_current[coord] = float4(scene, 1.0);
        return;
    }

    float2 texel = 1.0 / float2(texture_size(scene_texture));

    float3 scene_ycocg = rgb_to_ycocg(tonemap(scene));
    float3 moment_1 = scene_ycocg;
    float3 moment_2 = scene_ycocg * scene_ycocg;

    for (int y = -1; y <= 1; y++) {
        for (int x = -1; x <= 1; x++) {
            if (x == 0 && y == 0) {
                continue;
            }

            float3 neighbour = rgb_to_ycocg(tonemap(scene_texture.SampleLevel(linear_clamp, uv + float2(x, y) * texel, 0.0).rgb));

            moment_1 += neighbour;
            moment_2 += neighbour * neighbour;
        }
    }

    float3 mean = moment_1 / 9.0;
    float3 deviation = sqrt(max(moment_2 / 9.0 - mean * mean, float3(0.0)));

    float4 history_sample = graph_textures[push_constants.previous_history_descriptor_id].SampleLevel(linear_clamp, history_uv, 0.0);

    float3 history_ycocg = clamp(rgb_to_ycocg(tonemap(history_sample.rgb)), mean - deviation, mean + deviation);

    float motion_factor = clamp(motion_pixels / MOTION_BLEND_PIXELS, 0.0, 1.0);
    float accumulation_limit = lerp(ACCUMULATION_FRAMES_STILL, ACCUMULATION_FRAMES_MOVING, motion_factor);
    float accumulation = min(history_sample.a + 1.0, accumulation_limit);

    float3 result = inverse_tonemap(ycocg_to_rgb(lerp(history_ycocg, scene_ycocg, 1.0 / accumulation)));

    history_current[coord] = float4(result, accumulation);
}
