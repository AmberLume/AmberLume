import "../common.slang";
import "../projection.slang";
import "../readback.slang";

struct CascadeStatistics {
    float range_start;
    float range_end;
    float world_radius;
    float z_max;
};

struct CascadeStatisticsReadback {
    ReadbackHeader header;

    CascadeStatistics data[];
};

struct PushConstants {
    Ptr<Scene, Access.Read> scene_buffer_device_address;
    Ptr<Camera, Access.Read> camera_buffer_device_address;
    Ptr<DepthReduceResult, Access.Read> depth_reduce_result_buffer_device_address;
    Ptr<CullingView> culling_view_buffer_device_address;
    Ptr<ShadowCascade> shadow_cascades_buffer_device_address;
    Ptr<CascadeStatisticsReadback> cascade_statistics_buffer_device_address;

    uint cascade_count;
    uint shadow_resolution;

    float shadow_max_distance;
    float split_lambda;
    float shadow_caster_extension;
};

[[vk::push_constant]] ConstantBuffer<PushConstants> push_constants;

[vk::constant_id(0)] const uint GROUP_SIZE = 4;

static const float MIN_CASCADE_RANGE = 0.1;

static const float4x4 VULKAN_CORRECTION = float4x4(
    1.0, 0.0, 0.0, 0.0,
    0.0, -1.0, 0.0, 0.0,
    0.0, 0.0, 1.0, 0.0,
    0.0, 0.0, 0.0, 1.0
);

float4x4 look_at_rh(float3 eye, float3 target, float3 up) {
    float3 f = normalize(target - eye);
    float3 s = normalize(cross(f, up));
    float3 u = cross(s, f);

    return float4x4(
        float4(s, -dot(s, eye)),
        float4(u, -dot(u, eye)),
        float4(-f, dot(f, eye)),
        float4(0.0, 0.0, 0.0, 1.0)
    );
}

float4x4 ortho_rh(float left, float right, float bottom, float top, float near, float far) {
    float reciprocal_width = 1.0 / (right - left);
    float reciprocal_height = 1.0 / (top - bottom);
    float reciprocal_depth = 1.0 / (near - far);

    return float4x4(
        float4(reciprocal_width + reciprocal_width, 0.0, 0.0, -(left + right) * reciprocal_width),
        float4(0.0, reciprocal_height + reciprocal_height, 0.0, -(top + bottom) * reciprocal_height),
        float4(0.0, 0.0, reciprocal_depth, reciprocal_depth * near),
        float4(0.0, 0.0, 0.0, 1.0)
    );
}

[shader("compute")]
[numthreads(GROUP_SIZE, 1, 1)]
void main(uint3 local_invocation_id : SV_GroupThreadID) {
    uint cascade_index = local_invocation_id.x;
    if (cascade_index >= push_constants.cascade_count) {
        return;
    }

    Ptr<Camera, Access.Read> camera = push_constants.camera_buffer_device_address;

    float z_min = camera.near;
    float shadow_distance = min(push_constants.shadow_max_distance, camera.far);
    float farthest_depth = asfloat(push_constants.depth_reduce_result_buffer_device_address.farthest_depth_bits);
    float z_max_observed = linear_view_z(camera, farthest_depth);
    float z_max = z_max_observed > z_min + MIN_CASCADE_RANGE ? z_max_observed : shadow_distance;
    z_max = clamp(z_max, z_min + MIN_CASCADE_RANGE, shadow_distance);

    float cascade_count = float(push_constants.cascade_count);
    float lambda = push_constants.split_lambda;
    float ratio = z_max / z_min;
    float span = z_max - z_min;

    float t_start = float(cascade_index) / cascade_count;
    float t_end = float(cascade_index + 1u) / cascade_count;

    float range_start = lerp(z_min + span * t_start, z_min * pow(ratio, t_start), lambda);
    float range_end = lerp(z_min + span * t_end, z_min * pow(ratio, t_end), lambda);

    if (cascade_index == 0u) {
        range_start = z_min;
    }
    if (cascade_index == push_constants.cascade_count - 1u) {
        range_end = z_max;
    }

    float3 frustum_corners[8];
    int corner_index = 0;
    for (int x = 0; x < 2; x++) {
        for (int y = 0; y < 2; y++) {
            for (int z = 0; z < 2; z++) {
                float4 corner = mul(camera.inverse_view_projection, float4(float(x) * 2.0 - 1.0, float(y) * 2.0 - 1.0, float(z), 1.0));

                frustum_corners[corner_index++] = corner.xyz / corner.w;
            }
        }
    }

    float z_length = camera.far - camera.near;
    float t_param_start = (range_start - camera.near) / z_length;
    float t_param_end = (range_end - camera.near) / z_length;

    float3 cascade_corners[8];
    for (int j = 0; j < 4; j++) {
        float3 near_corner = frustum_corners[j * 2 + 1];
        float3 far_corner = frustum_corners[j * 2];
        float3 ray = far_corner - near_corner;

        cascade_corners[j * 2] = near_corner + ray * t_param_start;
        cascade_corners[j * 2 + 1] = near_corner + ray * t_param_end;
    }

    float3 center = float3(0.0);
    for (int j = 0; j < 8; j++) {
        center += cascade_corners[j];
    }
    center /= 8.0;

    float radius = 0.0;
    for (int j = 0; j < 8; j++) {
        radius = max(radius, distance(cascade_corners[j], center));
    }
    radius = ceil(radius);

    float3 light_direction = normalize(push_constants.scene_buffer_device_address.light_direction);
    float3 light_up = abs(dot(light_direction, float3(0.0, 1.0, 0.0))) > 0.99 ? float3(0.0, 0.0, 1.0) : float3(0.0, 1.0, 0.0);

    float4x4 light_view = look_at_rh(float3(0.0), light_direction, light_up);

    float3 light_space_center = mul(light_view, float4(center, 1.0)).xyz;
    float texel_size = 2.0 * radius / float(push_constants.shadow_resolution);
    light_space_center.x = floor(light_space_center.x / texel_size) * texel_size;
    light_space_center.y = floor(light_space_center.y / texel_size) * texel_size;

    float shadow_near = -(light_space_center.z + radius) - push_constants.shadow_caster_extension;
    float shadow_far = -(light_space_center.z - radius);

    float4x4 ortho = ortho_rh(
        light_space_center.x - radius, light_space_center.x + radius,
        light_space_center.y - radius, light_space_center.y + radius,
        shadow_near, shadow_far
    );
    float4x4 light_view_projection = mul(mul(VULKAN_CORRECTION, ortho), light_view);

    Ptr<ShadowCascade> cascade = push_constants.shadow_cascades_buffer_device_address + cascade_index;
    cascade.view_projection = light_view_projection;
    cascade.split = range_end;
    cascade.world_radius = radius;

    Ptr<CascadeStatisticsReadback> statistics = push_constants.cascade_statistics_buffer_device_address;

    InterlockedOr(statistics.header.written, 1);
    statistics.data[cascade_index].range_start = range_start;
    statistics.data[cascade_index].range_end = range_end;
    statistics.data[cascade_index].world_radius = radius;
    statistics.data[cascade_index].z_max = z_max;

    float4 planes[6];
    planes[0] = light_view_projection[3] + light_view_projection[0];
    planes[1] = light_view_projection[3] - light_view_projection[0];
    planes[2] = light_view_projection[3] + light_view_projection[1];
    planes[3] = light_view_projection[3] - light_view_projection[1];
    planes[4] = light_view_projection[2];
    planes[5] = light_view_projection[3] - light_view_projection[2];

    for (int p = 0; p < 6; p++) {
        push_constants.culling_view_buffer_device_address[cascade_index].frustum_planes[p] = planes[p] / length(planes[p].xyz);
    }
}
