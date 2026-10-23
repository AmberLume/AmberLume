import "../common.slang";
import "../bindings.slang";
import "../math.slang";
import "../sampling.slang";

struct PushConstants {
    Ptr<Camera, Access.Read> camera_buffer_device_address;

    uint view_z_descriptor_id;
    uint normal_descriptor_id;
    uint gtao_storage_id;
    uint width;
    uint height;
    uint temporal_index;
    uint view_z_mip_count;

    float radius;
    float power;
};

[[vk::push_constant]] ConstantBuffer<PushConstants> push_constants;

static const int SLICE_COUNT = 3;
static const int STEP_COUNT = 4;
static const float SAMPLE_DISTRIBUTION_POWER = 2.0;
static const float PIXEL_TOO_CLOSE = 1.3;
static const float MIP_SAMPLING_OFFSET = 3.0;

float load_view_z(int2 coord, int mip, int2 size) {
    int2 mip_size = max(size >> mip, int2(1));

    return graph_textures[push_constants.view_z_descriptor_id].Load(
        int3(clamp(coord >> mip, int2(0), mip_size - 1), mip)
    ).r;
}

float3 view_position(Ptr<Camera, Access.Read> camera, float2 uv, float view_z) {
    float2 view_xy = (uv * 2.0 - 1.0) * float2(camera.tan_half_fov.x, -camera.tan_half_fov.y) * view_z;

    return float3(view_xy, -view_z);
}

float fast_acos(float x) {
    float abs_x = abs(x);
    float res = -0.156583 * abs_x + HALF_PI;
    res *= sqrt(1.0 - abs_x);
    return x >= 0.0 ? res : PI - res;
}

[shader("compute")]
[numthreads(16, 16, 1)]
void main(uint3 global_invocation_id : SV_DispatchThreadID) {
    int2 coord = int2(global_invocation_id.xy);
    int2 size = int2(push_constants.width, push_constants.height);
    if (any(coord >= size)) {
        return;
    }

    Ptr<Camera, Access.Read> camera = push_constants.camera_buffer_device_address;

    float2 center_uv = (float2(coord) + 0.5) / float2(size);
    float center_z = load_view_z(coord, 0, size);

    if (center_z >= camera.far * 0.999) {
        storage_images_r16f[push_constants.gtao_storage_id][coord] = 1.0;
        return;
    }

    float3 position = view_position(camera, center_uv, center_z);
    float3 view = normalize(-position);

    float3 world_normal = normalize(graph_textures[push_constants.normal_descriptor_id].SampleLevel(samplers[SAMPLER_NEAREST_CLAMP], center_uv, 0.0).xyz);
    float3 normal = normalize(mul((float3x3)camera.view, world_normal));

    float focal_y = 1.0 / camera.tan_half_fov.y;
    float pixel_radius = push_constants.radius * focal_y / (2.0 * center_z) * float(size.y);
    pixel_radius = clamp(pixel_radius, float(STEP_COUNT), 0.2 * float(size.y));

    float2 noise = frac(float2(
        sample_seed(coord, 0u, push_constants.temporal_index),
        sample_seed(coord, 1u, push_constants.temporal_index)
    ));

    float visibility = 0.0;

    for (int slice = 0; slice < SLICE_COUNT; slice++) {
        float phi = PI * (float(slice) + noise.x) / float(SLICE_COUNT);
        float2 omega = float2(cos(phi), -sin(phi));

        float3 direction = float3(cos(phi), sin(phi), 0.0);
        float3 axis_raw = cross(direction, view);
        float axis_length = length(axis_raw);
        if (axis_length < 1e-4) {
            continue;
        }
        float3 axis = axis_raw / axis_length;

        float3 ortho_direction = direction - dot(direction, view) * view;
        float3 projected_normal = normal - axis * dot(normal, axis);
        float projected_normal_length = length(projected_normal);
        if (projected_normal_length < 1e-4) {
            continue;
        }

        float sign_n = sign(dot(ortho_direction, projected_normal));
        float cos_n = clamp(dot(projected_normal, view) / projected_normal_length, 0.0, 1.0);
        float angle_n = sign_n * fast_acos(cos_n);

        float horizon_cos_plus = -1.0;
        float horizon_cos_minus = -1.0;

        for (int side = 0; side < 2; side++) {
            float side_sign = side == 0 ? -1.0 : 1.0;
            float horizon_cos = -1.0;

            for (int step = 0; step < STEP_COUNT; step++) {
                float step_noise = frac(noise.y + float(slice + step * STEP_COUNT) * GOLDEN_RATIO);
                float t = (float(step) + step_noise) / float(STEP_COUNT);
                t = pow(t, SAMPLE_DISTRIBUTION_POWER);
                float march_pixels = pixel_radius * t + PIXEL_TOO_CLOSE;
                float2 sample_pixel = float2(coord) + 0.5 + side_sign * omega * march_pixels;
                int2 sample_coord = clamp(int2(floor(sample_pixel)), int2(0), size - 1);

                int mip = int(clamp(
                    floor(log2(march_pixels)) - MIP_SAMPLING_OFFSET,
                    0.0,
                    float(push_constants.view_z_mip_count - 1u)
                ));

                float sample_z = load_view_z(sample_coord, mip, size);
                float2 sample_uv = (float2(sample_coord) + 0.5) / float2(size);
                float3 sample_position = view_position(camera, sample_uv, sample_z);

                float3 delta = sample_position - position;
                float sample_distance = length(delta);
                if (sample_distance < 1e-4) {
                    continue;
                }

                float3 sample_direction = delta / sample_distance;
                float cos_horizon = dot(sample_direction, view);
                float falloff = 1.0 - smoothstep(0.5 * push_constants.radius, push_constants.radius, sample_distance);
                float weighted = lerp(-1.0, cos_horizon, falloff);

                horizon_cos = max(horizon_cos, weighted);
            }

            if (side == 0) {
                horizon_cos_minus = horizon_cos;
            } else {
                horizon_cos_plus = horizon_cos;
            }
        }

        float horizon_high = fast_acos(clamp(horizon_cos_plus, -1.0, 1.0));
        float horizon_low = -fast_acos(clamp(horizon_cos_minus, -1.0, 1.0));
        horizon_low = angle_n + max(horizon_low - angle_n, -HALF_PI);
        horizon_high = angle_n + min(horizon_high - angle_n, HALF_PI);

        float arc =
            0.25 * (-cos(2.0 * horizon_low - angle_n) + cos_n + 2.0 * horizon_low * sin(angle_n)) +
            0.25 * (-cos(2.0 * horizon_high - angle_n) + cos_n + 2.0 * horizon_high * sin(angle_n));

        visibility += projected_normal_length * arc;
    }

    visibility = clamp(visibility / float(SLICE_COUNT), 0.0, 1.0);
    visibility = pow(visibility, push_constants.power);

    storage_images_r16f[push_constants.gtao_storage_id][coord] = visibility;
}
