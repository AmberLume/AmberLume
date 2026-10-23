import "../common.slang";
import "../bindings.slang";
import "../projection.slang";
import "../ray_tracing.slang";
import "../sampling.slang";
import "../texture.slang";

struct PushConstants {
    Ptr<Scene, Access.Read> scene_buffer_device_address;
    Ptr<Camera, Access.Read> camera_buffer_device_address;
    Ptr<Entity, Access.Read> entity_buffer_device_address;
    Ptr<Mesh, Access.Read> mesh_buffer_device_address;
    Ptr<Submesh, Access.Read> submesh_buffer_device_address;
    Ptr<Material, Access.Read> material_buffer_device_address;

    uint depth_descriptor_id;
    uint normal_descriptor_id;
    uint transmittance_storage_id;
    uint tlas_descriptor_id;

    float sun_angular_radius;
    uint sample_count;
    uint frame_number;
};

[[vk::push_constant]] ConstantBuffer<PushConstants> push_constants;

typealias TransmittanceQuery = RayQuery<RAY_FLAG_FORCE_NON_OPAQUE>;

static const float3 LUMINANCE_WEIGHTS = float3(0.2126, 0.7152, 0.0722);

Material candidate_material(TransmittanceQuery query) {
    Entity entity = push_constants.entity_buffer_device_address[query.CandidateInstanceID()];
    Mesh mesh = push_constants.mesh_buffer_device_address[entity.mesh_index];
    Submesh submesh = push_constants.submesh_buffer_device_address[mesh.submesh_offset + query.CandidateGeometryIndex()];

    return push_constants.material_buffer_device_address[submesh.material_index];
}

float3 blend_transmittance(Material material) {
    float4 base_color = material.base_color_factor;

    float max_channel = max(base_color.r, max(base_color.g, base_color.b));
    float3 hue = max_channel > 0.0 ? base_color.rgb / max_channel : float3(1.0);

    float hue_luminance = dot(hue, LUMINANCE_WEIGHTS);
    float saturation = hue_luminance < 1.0
        ? min(base_color.a / (1.0 - hue_luminance), 1.0)
        : 0.0;

    float3 tint = lerp(float3(1.0), hue, saturation);

    return (1.0 - base_color.a) * tint / dot(tint, LUMINANCE_WEIGHTS);
}

float3 trace_transmittance(RaytracingAccelerationStructure acceleration_structure, float3 origin, float3 direction, float t_min) {
    RayDesc ray;
    ray.Origin = origin;
    ray.TMin = t_min;
    ray.Direction = direction;
    ray.TMax = RAY_T_MAX;

    TransmittanceQuery query;
    query.TraceRayInline(acceleration_structure, RAY_FLAG_NONE, RT_INSTANCE_MASK_OPAQUE | RT_INSTANCE_MASK_BLEND, ray);

    float3 transmittance = float3(1.0);

    while (query.Proceed()) {
        Material material = candidate_material(query);

        if ((material.flags & MATERIAL_ALPHA_MODE_BITS) != MATERIAL_FLAG_ALPHA_BLEND) {
            query.Abort();

            return float3(0.0);
        }

        if (!query.CandidateTriangleFrontFace()) {
            continue;
        }

        transmittance *= blend_transmittance(material);
    }

    return transmittance;
}

[shader("compute")]
[numthreads(16, 16, 1)]
void main(uint3 global_invocation_id : SV_DispatchThreadID) {
    int2 coord = int2(global_invocation_id.xy);
    int2 size = texture_size(storage_images_rgba16f[push_constants.transmittance_storage_id]);
    if (any(coord >= size)) {
        return;
    }

    float depth = graph_textures[push_constants.depth_descriptor_id].Load(int3(coord, 0)).r;

    float3 transmittance = float3(1.0);
    if (depth > 0.0) {
        float3 normal = normalize(graph_textures[push_constants.normal_descriptor_id].Load(int3(coord, 0)).xyz);
        Ptr<Camera, Access.Read> camera = push_constants.camera_buffer_device_address;

        float3 sun_direction = normalize(-push_constants.scene_buffer_device_address.light_direction);

        if (dot(normal, sun_direction) <= 0.0) {
            transmittance = float3(0.0);
        } else {
            RaytracingAccelerationStructure acceleration_structure = acceleration_structures[push_constants.tlas_descriptor_id];

            float3 world_position = world_position_from_depth(camera.inverse_view_projection, camera.jitter, coord, size, depth);
            float ray_t_min = max(RAY_T_MIN, pixel_footprint(camera, world_position, float(size.y)));

            uint samples = max(push_constants.sample_count, 1u);
            float3 transmitted = float3(0.0);
            for (uint i = 0u; i < samples; i++) {
                float3 direction = sun_disk_direction(sun_direction, push_constants.sun_angular_radius, coord, i, samples, push_constants.frame_number);

                transmitted += trace_transmittance(acceleration_structure, world_position, direction, ray_t_min);
            }
            transmittance = transmitted / float(samples);
        }
    }

    storage_images_rgba16f[push_constants.transmittance_storage_id][coord] = float4(transmittance, 0.0);
}
