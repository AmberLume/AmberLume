import "../common.slang";
import "../bindings.slang";

struct PushConstants {
    Ptr<DepthReduceResult> result_buffer_device_address;

    uint depth_descriptor_id;
    uint depth_width;
    uint depth_height;
    uint stride;
};

[[vk::push_constant]] ConstantBuffer<PushConstants> push_constants;

groupshared uint group_min;

[shader("compute")]
[numthreads(16, 16, 1)]
void main(uint3 global_invocation_id : SV_DispatchThreadID, uint local_invocation_index : SV_GroupIndex) {
    if (local_invocation_index == 0u) {
        group_min = asuint(1.0);
    }
    GroupMemoryBarrierWithGroupSync();

    uint2 coord = global_invocation_id.xy * push_constants.stride;
    float depth = 1.0;
    if (all(coord < uint2(push_constants.depth_width, push_constants.depth_height))) {
        float sampled = graph_textures[push_constants.depth_descriptor_id].Load(int3(int2(coord), 0)).r;
        if (sampled > 0.0) {
            depth = sampled;
        }
    }

    InterlockedMin(group_min, asuint(depth));
    GroupMemoryBarrierWithGroupSync();

    if (local_invocation_index == 0u) {
        InterlockedMin(push_constants.result_buffer_device_address.farthest_depth_bits, group_min);
    }
}
