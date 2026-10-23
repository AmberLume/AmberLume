import "../common.slang";
import "../bindings.slang";
import "../math.slang";
import "../projection.slang";
import "../sampling.slang";
import "../texture.slang";

struct PushConstants {
    Ptr<Scene, Access.Read> scene_buffer_device_address;
    Ptr<Camera, Access.Read> camera_buffer_device_address;
    Ptr<ShadowCascade, Access.Read> shadow_cascades_buffer_device_address;

    uint depth_descriptor_id;
    uint normal_descriptor_id;
    uint shadow_array_descriptor_id;
    uint output_storage_id;

    uint shadow_pcf_sample_count;
    uint frame_index;

    float shadow_bias;
    float shadow_normal_bias;
    float shadow_pcf_world_radius;
    float shadow_cascade_blend_range;
};

[[vk::push_constant]] ConstantBuffer<PushConstants> push_constants;

static const float SILVER_RATIO = 0.4142135623730950488;
static const uint MAX_PCF_SAMPLES = 16u;
static const uint TEMPORAL_PERIOD = 64u;

float interleaved_gradient_noise(int2 coord) {
    return frac(52.9829189 * frac(0.06711056 * (float(coord.x) + 0.5) + 0.00583715 * (float(coord.y) + 0.5)));
}

float sample_cascade(
    Ptr<ShadowCascade, Access.Read> cascades,
    uint cascade_index,
    float3 shadow_position,
    float3 geometry_normal,
    float angle,
    float radial_offset
) {
    float4x4 light_matrix = cascades[cascade_index].view_projection;
    float4 light_clip = mul(light_matrix, float4(shadow_position, 1.0));
    float3 light_ndc = light_clip.xyz / light_clip.w;
    float2 shadow_uv = light_ndc.xy * 0.5 + 0.5;
    float receiver_z = light_ndc.z;

    if (any(shadow_uv < 0.0) || any(shadow_uv > 1.0)) {
        return 1.0;
    }

    Texture2DArray shadow_array = graph_texture_arrays[push_constants.shadow_array_descriptor_id];
    SamplerComparisonState shadow_sampler = comparison_samplers[SAMPLER_SHADOW];

    int sample_count = int(min(push_constants.shadow_pcf_sample_count, MAX_PCF_SAMPLES));
    if (push_constants.shadow_pcf_world_radius <= 0.0 || sample_count <= 1) {
        return shadow_array.SampleCmpLevelZero(shadow_sampler, float3(shadow_uv, float(cascade_index)), receiver_z);
    }

    float3x3 light_to_world = inverse((float3x3)light_matrix);
    float3 world_per_u = mul(light_to_world, float3(2.0, 0.0, 0.0));
    float3 world_per_v = mul(light_to_world, float3(0.0, 2.0, 0.0));
    float3 world_per_z = mul(light_to_world, float3(0.0, 0.0, 1.0));

    float2 depth_slope = -float2(dot(geometry_normal, world_per_u), dot(geometry_normal, world_per_v))
        / dot(geometry_normal, world_per_z);

    float uv_radius = push_constants.shadow_pcf_world_radius / (2.0 * cascades[cascade_index].world_radius);
    float sum = 0.0;
    for (int i = 0; i < sample_count; i++) {
        float r = sqrt((float(i) + radial_offset) / float(sample_count));
        float theta = float(i) * GOLDEN_ANGLE + angle;
        float2 offset = r * float2(cos(theta), sin(theta)) * uv_radius;
        float sample_z = receiver_z + dot(offset, depth_slope);
        sum += shadow_array.SampleCmpLevelZero(shadow_sampler, float3(shadow_uv + offset, float(cascade_index)), sample_z);
    }
    return sum / float(sample_count);
}

[shader("compute")]
[numthreads(16, 16, 1)]
void main(uint3 global_invocation_id : SV_DispatchThreadID) {
    int2 coord = int2(global_invocation_id.xy);
    int2 size = texture_size(storage_images_r16f[push_constants.output_storage_id]);
    if (any(coord >= size)) {
        return;
    }

    float depth = graph_textures[push_constants.depth_descriptor_id].Load(int3(coord, 0)).r;
    if (depth <= 0.0) {
        storage_images_r16f[push_constants.output_storage_id][coord] = 1.0;
        return;
    }

    Ptr<Scene, Access.Read> scene = push_constants.scene_buffer_device_address;
    Ptr<Camera, Access.Read> camera = push_constants.camera_buffer_device_address;
    Ptr<ShadowCascade, Access.Read> cascades = push_constants.shadow_cascades_buffer_device_address;

    float3 world_position = world_position_from_depth(camera.inverse_view_projection, camera.jitter, coord, size, depth);
    float3 geometry_normal = normalize(graph_textures[push_constants.normal_descriptor_id].Load(int3(coord, 0)).xyz);
    float view_z = linear_view_z(camera, depth);

    uint cascade_count = scene.shadow_cascade_count;
    uint cascade_index = 0u;
    for (uint i = 0u; i < cascade_count - 1u; ++i) {
        if (view_z > cascades[i].split) {
            cascade_index = i + 1u;
        }
    }

    float3 sun_direction = normalize(-scene.light_direction);
    float n_dot_l = clamp(dot(geometry_normal, sun_direction), 0.0, 1.0);

    float3 shadow_position = world_position
        + geometry_normal * push_constants.shadow_normal_bias * (1.0 - n_dot_l)
        + sun_direction * push_constants.shadow_bias;

    float noise = interleaved_gradient_noise(coord);
    float rotation_phase = float(push_constants.frame_index % TEMPORAL_PERIOD);
    float angle = frac(noise + rotation_phase * GOLDEN_RATIO) * TWO_PI;
    float radial_offset = frac(noise + rotation_phase * SILVER_RATIO);

    float shadow = sample_cascade(cascades, cascade_index, shadow_position, geometry_normal, angle, radial_offset);

    float blend_range = push_constants.shadow_cascade_blend_range;
    if (blend_range > 0.0 && cascade_index + 1u < cascade_count) {
        float current_split = cascades[cascade_index].split;
        float previous_split = cascade_index > 0u ? cascades[cascade_index - 1u].split : camera.near;
        float fade_start = current_split - (current_split - previous_split) * blend_range;
        float fade = smoothstep(fade_start, current_split, view_z);
        if (fade > 0.0) {
            float next_shadow = sample_cascade(cascades, cascade_index + 1u, shadow_position, geometry_normal, angle, radial_offset);
            shadow = lerp(shadow, next_shadow, fade);
        }
    }

    storage_images_r16f[push_constants.output_storage_id][coord] = shadow;
}
