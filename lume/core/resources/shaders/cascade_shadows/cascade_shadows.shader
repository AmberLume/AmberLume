import "../common.slang";

struct PushConstants {
    Ptr<DrawData, Access.Read> draw_data_buffer_device_address;
    Ptr<Entity, Access.Read> entity_buffer_device_address;
    Ptr<VertexPosition, Access.Read> vertex_position_buffer_device_address;
    Ptr<ShadowCascade, Access.Read> shadow_cascades_buffer_device_address;
};

[[vk::push_constant]] ConstantBuffer<PushConstants> push_constants;

static const float4 CULLED_VERTEX_POSITION = float4(2.0, 2.0, 2.0, 1.0);

bool is_cascade_visible(uint cascade_mask, uint view_index) {
    return (cascade_mask & (1u << view_index)) != 0u;
}

[shader("vertex")]
float4 main(uint vertex_index : SV_VulkanVertexID, uint draw_id : SV_VulkanInstanceID, uint view_index : SV_ViewID) : SV_Position {
    DrawData draw_data = push_constants.draw_data_buffer_device_address[draw_id];

    if (!is_cascade_visible(draw_data.cascade_mask, view_index)) {
        return CULLED_VERTEX_POSITION;
    }

    Entity entity = push_constants.entity_buffer_device_address[draw_data.entity_index];
    float4 world_position = mul(entity.transform_matrix, float4(push_constants.vertex_position_buffer_device_address[vertex_index].position, 1.0));

    return mul(push_constants.shadow_cascades_buffer_device_address[view_index].view_projection, world_position);
}
