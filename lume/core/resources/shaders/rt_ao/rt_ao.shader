import "../common.slang";
import "../bindings.slang";
import "../projection.slang";
import "../ray_tracing.slang";
import "../sampling.slang";

struct PushConstants {
    Ptr<Camera, Access.Read> camera_buffer_device_address;

    uint depth_descriptor_id;
    uint normal_descriptor_id;
    uint ao_storage_id;
    uint width;
    uint height;
    uint tlas_descriptor_id;

    float ao_radius;
    uint sample_count;
    float ao_power;
    uint frame_number;
};

[[vk::push_constant]] ConstantBuffer<PushConstants> push_constants;

[shader("compute")]
[numthreads(16, 16, 1)]
void main(uint3 global_invocation_id : SV_DispatchThreadID) {
    int2 coord = int2(global_invocation_id.xy);
    int2 size = int2(push_constants.width, push_constants.height);
    if (any(coord >= size)) {
        return;
    }

    float depth = graph_textures[push_constants.depth_descriptor_id].Load(int3(coord, 0)).r;

    float ao = 1.0;
    if (depth > 0.0) {
        Ptr<Camera, Access.Read> camera = push_constants.camera_buffer_device_address;

        float3 world_position = world_position_from_depth(camera.inverse_view_projection, camera.jitter, coord, size, depth);
        float footprint = pixel_footprint(camera, world_position, float(size.y));

        float radius = max(push_constants.ao_radius, 1e-6);
        float resolvable = clamp(2.0 - 2.0 * footprint / radius, 0.0, 1.0);

        if (resolvable > 0.0) {
            RaytracingAccelerationStructure acceleration_structure = acceleration_structures[push_constants.tlas_descriptor_id];
            float3 normal = normalize(graph_textures[push_constants.normal_descriptor_id].Load(int3(coord, 0)).xyz);

            uint samples = max(push_constants.sample_count, 1u);
            uint occluded = 0u;
            for (uint i = 0u; i < samples; i++) {
                float3 direction = cosine_hemisphere_direction(normal, coord, i, samples, push_constants.frame_number);

                if (trace_occluded(acceleration_structure, world_position, direction, RAY_T_MIN, radius)) {
                    occluded++;
                }
            }

            ao = 1.0 - float(occluded) / float(samples);
            ao = pow(ao, push_constants.ao_power);
            ao = lerp(1.0, ao, resolvable);
        }
    }

    storage_images_r16f[push_constants.ao_storage_id][coord] = ao;
}
