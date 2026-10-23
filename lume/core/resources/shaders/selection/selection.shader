import "../common.slang";
import "../bindings.slang";
import "../fullscreen.slang";
import "../texture.slang";

struct PushConstants {
    Ptr<Camera, Access.Read> camera_buffer_device_address;
    Ptr<EntityOutline, Access.Read> entity_outline_buffer_device_address;

    float2 entity_id_texel_scale;

    uint entity_id_descriptor_id;
    uint mask_descriptor_id;

    int radius;
    int mask_scale;
};

[[vk::push_constant]] ConstantBuffer<PushConstants> push_constants;

static const uint ENTITY_NONE = 0xffffffffu;

uint entity_at(int2 coord, int2 size) {
    return graph_utextures[push_constants.entity_id_descriptor_id].Load(int3(clamp(coord, int2(0), size - 1), 0));
}

bool is_outlined(uint entity_index) {
    return entity_index != ENTITY_NONE && push_constants.entity_outline_buffer_device_address[entity_index].outline.a > 0.0;
}

bool outline_nearby(int2 coord) {
    Texture2D<uint> mask = graph_utextures[push_constants.mask_descriptor_id];
    int2 mask_size = texture_size(mask);

    int2 first = clamp((coord - push_constants.radius) / push_constants.mask_scale, int2(0), mask_size - 1);
    int2 last = clamp((coord + push_constants.radius) / push_constants.mask_scale, int2(0), mask_size - 1);

    for (int y = first.y; y <= last.y; y++) {
        for (int x = first.x; x <= last.x; x++) {
            if (mask.Load(int3(x, y, 0)) != 0u) {
                return true;
            }
        }
    }

    return false;
}

[shader("vertex")]
float4 main(uint vertex_index : SV_VulkanVertexID) : SV_Position {
    return fullscreen_position(vertex_index);
}

[shader("fragment")]
float4 main(float4 position : SV_Position) : SV_Target0 {
    int2 entity_id_size = texture_size(graph_utextures[push_constants.entity_id_descriptor_id]);
    int2 coord = clamp(int2(position.xy * push_constants.entity_id_texel_scale), int2(0), entity_id_size - 1);

    float2 jitter = push_constants.camera_buffer_device_address.jitter * float2(entity_id_size) * 0.5;

    if (is_outlined(entity_at(coord, entity_id_size))) {
        return float4(0.0);
    }

    if (!outline_nearby(coord)) {
        return float4(0.0);
    }

    int radius = push_constants.radius;

    float nearest = float((radius + 1) * (radius + 1));
    uint nearest_entity = ENTITY_NONE;

    for (int y = -radius; y <= radius; y++) {
        for (int x = -radius; x <= radius; x++) {
            float2 delta = float2(x, y) - jitter;
            float squared = dot(delta, delta);

            if (squared >= nearest) {
                continue;
            }

            uint entity_index = entity_at(coord + int2(x, y), entity_id_size);

            if (!is_outlined(entity_index)) {
                continue;
            }

            nearest = squared;
            nearest_entity = entity_index;
        }
    }

    if (nearest_entity == ENTITY_NONE) {
        return float4(0.0);
    }

    float4 color = push_constants.entity_outline_buffer_device_address[nearest_entity].outline;

    float falloff = 1.0 - clamp(sqrt(nearest) / float(radius), 0.0, 1.0);

    return float4(color.rgb, color.a * falloff);
}
