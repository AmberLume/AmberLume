import "../common.slang";
import "../bindings.slang";
import "../projection.slang";

struct PushConstants {
    Ptr<Camera, Access.Read> camera_buffer_device_address;

    uint depth_descriptor_id;
    uint normal_descriptor_id;
    uint guide_storage_id;
    uint width;
    uint height;
};

[[vk::push_constant]] ConstantBuffer<PushConstants> push_constants;

[shader("compute")]
[numthreads(16, 16, 1)]
void main(uint3 global_invocation_id : SV_DispatchThreadID) {
    int2 coord = int2(global_invocation_id.xy);
    if (coord.x >= int(push_constants.width) || coord.y >= int(push_constants.height)) {
        return;
    }

    Ptr<Camera, Access.Read> camera = push_constants.camera_buffer_device_address;

    float depth = graph_textures[push_constants.depth_descriptor_id].Load(int3(coord, 0)).r;

    float4 guide = float4(0.0);
    if (depth > 0.0) {
        float3 world_position = world_position_from_depth(
            camera.inverse_view_projection,
            camera.jitter,
            coord,
            int2(int(push_constants.width), int(push_constants.height)),
            depth
        );

        float linear_z = distance(world_position, camera.position);
        float3 normal = normalize(graph_textures[push_constants.normal_descriptor_id].Load(int3(coord, 0)).xyz);

        guide = float4(linear_z, normal);
    }

    storage_images_rgba16f[push_constants.guide_storage_id][coord] = guide;
}
