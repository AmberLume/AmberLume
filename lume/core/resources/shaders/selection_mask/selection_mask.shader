import "../common.slang";
import "../bindings.slang";
import "../texture.slang";

struct PushConstants {
    Ptr<EntityOutline, Access.Read> entity_outline_buffer_device_address;

    uint entity_id_descriptor_id;
    uint mask_storage_id;
    uint width;
    uint height;

    int mask_scale;
};

[[vk::push_constant]] ConstantBuffer<PushConstants> push_constants;

static const uint ENTITY_NONE = 0xffffffffu;

[shader("compute")]
[numthreads(16, 16, 1)]
void main(uint3 global_invocation_id : SV_DispatchThreadID) {
    int2 coord = int2(global_invocation_id.xy);
    int2 size = int2(push_constants.width, push_constants.height);
    if (any(coord >= size)) {
        return;
    }

    Texture2D<uint> entity_id = graph_utextures[push_constants.entity_id_descriptor_id];
    int2 entity_id_size = texture_size(entity_id);

    uint outlined = 0u;

    for (int y = 0; y < push_constants.mask_scale; y++) {
        for (int x = 0; x < push_constants.mask_scale; x++) {
            int2 tap = min(coord * push_constants.mask_scale + int2(x, y), entity_id_size - 1);
            uint entity_index = entity_id.Load(int3(tap, 0));

            if (entity_index == ENTITY_NONE) {
                continue;
            }

            if (push_constants.entity_outline_buffer_device_address[entity_index].outline.a > 0.0) {
                outlined = 1u;
            }
        }
    }

    storage_images_r32ui[push_constants.mask_storage_id][coord] = outlined;
}
