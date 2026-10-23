import "../common.slang";

struct PhysicsDebugVertex {
    float3 point;

    uint _pad0;

    float4 color;
};

struct PushConstants {
    Ptr<Camera, Access.Read> camera_buffer_device_address;
    Ptr<PhysicsDebugVertex, Access.Read> physics_debug_vertex_buffer_device_address;
};

[[vk::push_constant]] ConstantBuffer<PushConstants> push_constants;

struct VertexOutput {
    float4 position : SV_Position;
    float4 color : COLOR;
};

[shader("vertex")]
VertexOutput main(uint vertex_index : SV_VulkanVertexID) {
    PhysicsDebugVertex vertex = push_constants.physics_debug_vertex_buffer_device_address[vertex_index];

    VertexOutput output;
    output.position = mul(push_constants.camera_buffer_device_address.view_projection, float4(vertex.point, 1.0));
    output.color = vertex.color;

    return output;
}

[shader("fragment")]
float4 main(VertexOutput input) : SV_Target0 {
    return input.color;
}
