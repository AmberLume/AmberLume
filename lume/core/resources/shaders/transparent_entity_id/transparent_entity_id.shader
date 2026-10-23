import "../common.slang";
import "../mesh_vertex.slang";
import "../projection.slang";

struct PushConstants {
    Ptr<Camera, Access.Read> camera_buffer_device_address;
    Ptr<DrawData, Access.Read> draw_data_buffer_device_address;
    Ptr<Entity, Access.Read> entity_buffer_device_address;
    Ptr<VertexPosition, Access.Read> vertex_position_buffer_device_address;
};

[[vk::push_constant]] ConstantBuffer<PushConstants> push_constants;

struct VertexOutput {
    float4 position : SV_Position;
    nointerpolation uint entity_index : ENTITY_INDEX;
    float4 current_clip : CURRENT_CLIP;
    float4 previous_clip : PREVIOUS_CLIP;
};

struct FragmentOutput {
    uint entity_index : SV_Target0;
    float2 velocity : SV_Target1;
};

[shader("vertex")]
VertexOutput main(uint vertex_index : SV_VulkanVertexID, uint draw_id : SV_VulkanInstanceID) {
    Ptr<Camera, Access.Read> camera = push_constants.camera_buffer_device_address;
    DrawData draw_data = push_constants.draw_data_buffer_device_address[draw_id];
    Entity entity = push_constants.entity_buffer_device_address[draw_data.entity_index];

    MeshClipPositions clip = mesh_clip_positions(camera, push_constants.vertex_position_buffer_device_address, entity, draw_data, vertex_index);

    VertexOutput output;
    output.position = jitter_clip_position(clip.current, camera.jitter);
    output.entity_index = draw_data.entity_index;
    output.current_clip = clip.current;
    output.previous_clip = clip.previous;

    return output;
}

[shader("fragment")]
FragmentOutput main(VertexOutput input) {
    FragmentOutput output;
    output.entity_index = input.entity_index;
    output.velocity = screen_velocity(input.current_clip, input.previous_clip);

    return output;
}
