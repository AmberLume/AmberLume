import "../bindings.slang";

struct PushConstants {
    uint source_descriptor_id;
    uint view_z_storage_id;
    uint width;
    uint height;
    uint source_width;
    uint source_height;

    float radius;
};

[[vk::push_constant]] ConstantBuffer<PushConstants> push_constants;

static const float FALLOFF_RANGE = 0.615;
static const float DEPTH_RANGE_SCALE = 0.75;

float filtered_depth(float4 depths) {
    float furthest = max(max(depths.x, depths.y), max(depths.z, depths.w));

    float effect_radius = DEPTH_RANGE_SCALE * push_constants.radius;
    float falloff_range = FALLOFF_RANGE * effect_radius;
    float falloff_from = effect_radius * (1.0 - FALLOFF_RANGE);

    float4 weights = clamp(
        (float4(furthest) - depths) * (-1.0 / falloff_range) + (falloff_from / falloff_range + 1.0),
        0.0,
        1.0
    );

    float total = weights.x + weights.y + weights.z + weights.w;

    return total > 0.0 ? dot(weights, depths) / total : furthest;
}

[shader("compute")]
[numthreads(16, 16, 1)]
void main(uint3 global_invocation_id : SV_DispatchThreadID) {
    int2 coord = int2(global_invocation_id.xy);
    int2 size = int2(int(push_constants.width), int(push_constants.height));

    if (coord.x >= size.x || coord.y >= size.y) {
        return;
    }

    int2 source_size = int2(int(push_constants.source_width), int(push_constants.source_height));
    int2 source_coord = coord * 2;

    float4 depths = float4(
        graph_textures[push_constants.source_descriptor_id].Load(int3(min(source_coord, source_size - 1), 0)).r,
        graph_textures[push_constants.source_descriptor_id].Load(int3(min(source_coord + int2(1, 0), source_size - 1), 0)).r,
        graph_textures[push_constants.source_descriptor_id].Load(int3(min(source_coord + int2(0, 1), source_size - 1), 0)).r,
        graph_textures[push_constants.source_descriptor_id].Load(int3(min(source_coord + int2(1, 1), source_size - 1), 0)).r
    );

    storage_images_r16f[push_constants.view_z_storage_id][coord] = filtered_depth(depths);
}
