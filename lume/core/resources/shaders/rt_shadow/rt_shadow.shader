import "../common.slang";
import "../bindings.slang";
import "../projection.slang";
import "../ray_tracing.slang";
import "../sampling.slang";
import "../texture.slang";

struct PushConstants {
    Ptr<Scene, Access.Read> scene_buffer_device_address;
    Ptr<Camera, Access.Read> camera_buffer_device_address;

    uint depth_descriptor_id;
    uint normal_descriptor_id;
    uint visibility_storage_id;
    uint tlas_descriptor_id;

    float sun_angular_radius;
    uint sample_count;
    uint frame_number;
};

[[vk::push_constant]] ConstantBuffer<PushConstants> push_constants;

[shader("compute")]
[numthreads(16, 16, 1)]
void main(uint3 global_invocation_id : SV_DispatchThreadID) {
    int2 coord = int2(global_invocation_id.xy);
    int2 size = texture_size(storage_images_r16f[push_constants.visibility_storage_id]);
    if (any(coord >= size)) {
        return;
    }

    float depth = graph_textures[push_constants.depth_descriptor_id].Load(int3(coord, 0)).r;

    float visibility = 1.0;
    if (depth > 0.0) {
        float3 normal = normalize(graph_textures[push_constants.normal_descriptor_id].Load(int3(coord, 0)).xyz);
        Ptr<Camera, Access.Read> camera = push_constants.camera_buffer_device_address;

        float3 sun_direction = normalize(-push_constants.scene_buffer_device_address.light_direction);

        if (dot(normal, sun_direction) <= 0.0) {
            visibility = 0.0;
        } else {
            RaytracingAccelerationStructure acceleration_structure = acceleration_structures[push_constants.tlas_descriptor_id];

            float3 world_position = world_position_from_depth(camera.inverse_view_projection, camera.jitter, coord, size, depth);
            float ray_t_min = max(RAY_T_MIN, pixel_footprint(camera, world_position, float(size.y)));

            uint samples = max(push_constants.sample_count, 1u);
            float lit = 0.0;
            for (uint i = 0u; i < samples; i++) {
                float3 direction = sun_disk_direction(sun_direction, push_constants.sun_angular_radius, coord, i, samples, push_constants.frame_number);

                if (!trace_occluded(acceleration_structure, world_position, direction, ray_t_min, RAY_T_MAX)) {
                    lit += 1.0;
                }
            }
            visibility = lit / float(samples);
        }
    }

    storage_images_r16f[push_constants.visibility_storage_id][coord] = visibility;
}
