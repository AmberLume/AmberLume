import "../common.slang";
import "../bindings.slang";
import "../sampling.slang";

struct PushConstants {
    Ptr<Camera, Access.Read> camera_buffer_device_address;

    uint noisy_descriptor_id;
    uint guide_descriptor_id;
    uint ao_storage_id;
    uint width;
    uint height;

    float plane_sensitivity;
    float normal_threshold;
    float blur_radius;

    uint frame_number;
};

[[vk::push_constant]] ConstantBuffer<PushConstants> push_constants;

static const int TAP_COUNT = 8;

float3 view_position(Ptr<Camera, Access.Read> camera, float2 uv, float linear_z) {
    float3 ray = float3((uv * 2.0 - 1.0) * float2(camera.tan_half_fov.x, -camera.tan_half_fov.y), -1.0);

    return normalize(ray) * linear_z;
}

[shader("compute")]
[numthreads(16, 16, 1)]
void main(uint3 global_invocation_id : SV_DispatchThreadID) {
    int2 coord = int2(global_invocation_id.xy);
    int2 size = int2(push_constants.width, push_constants.height);
    if (any(coord >= size)) {
        return;
    }

    Texture2D noisy = graph_textures[push_constants.noisy_descriptor_id];
    Texture2D guide = graph_textures[push_constants.guide_descriptor_id];

    float center = noisy.Load(int3(coord, 0)).r;
    float4 center_guide = guide.Load(int3(coord, 0));

    float linear_z = center_guide.r;
    float3 normal = center_guide.gba;

    float ao = center;

    if (linear_z > 0.0) {
        Ptr<Camera, Access.Read> camera = push_constants.camera_buffer_device_address;

        float2 uv = (float2(coord) + 0.5) / float2(size);
        float3 position = view_position(camera, uv, linear_z);
        float3 view_normal = mul((float3x3)camera.view, normal);

        float frustum_size = 2.0 * camera.tan_half_fov.y * linear_z;

        float total = center;
        float total_weight = 1.0;

        for (int tap = 0; tap < TAP_COUNT; tap++) {
            float2 offset = vogel_disk_sample(push_constants.blur_radius, TAP_COUNT, sample_seed(coord, uint(tap), push_constants.frame_number));
            int2 tap_coord = coord + int2(round(offset));

            if (all(tap_coord == coord) || any(tap_coord < 0) || any(tap_coord >= size)) {
                continue;
            }

            float4 tap_guide = guide.Load(int3(tap_coord, 0));

            if (tap_guide.r <= 0.0) {
                continue;
            }

            float2 tap_uv = (float2(tap_coord) + 0.5) / float2(size);
            float3 tap_position = view_position(camera, tap_uv, tap_guide.r);

            float plane_distance = abs(dot(view_normal, tap_position - position));
            float plane_weight = 1.0 - smoothstep(0.0, 1.0, plane_distance / (push_constants.plane_sensitivity * frustum_size));
            float normal_weight = max(0.0, (dot(tap_guide.gba, normal) - push_constants.normal_threshold) / (1.0 - push_constants.normal_threshold));

            float weight = plane_weight * normal_weight;

            if (weight <= 0.0) {
                continue;
            }

            total += weight * noisy.Load(int3(tap_coord, 0)).r;
            total_weight += weight;
        }

        ao = total / total_weight;
    }

    storage_images_r16f[push_constants.ao_storage_id][coord] = ao;
}
