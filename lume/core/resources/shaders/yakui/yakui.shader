import "../bindings.slang";

struct UiVertex {
    float2 position;
    float2 texture_coordinates;
    float4 color;
};

enum RenderMode : uint {
    Solid = 0,
    Texture = 1,
};

struct PushConstants {
    Ptr<UiVertex, Access.Read> ui_vertex_buffer_device_address;

    uint texture_index;
    RenderMode render_mode;
};

[[vk::push_constant]] ConstantBuffer<PushConstants> push_constants;

struct VertexOutput {
    float4 position : SV_Position;
    float4 color : COLOR;
    float2 texture_coordinates : TEXTURE_COORDINATES;
};

[shader("vertex")]
VertexOutput main(uint vertex_index : SV_VulkanVertexID) {
    UiVertex ui_vertex = push_constants.ui_vertex_buffer_device_address[vertex_index];

    VertexOutput output;
    output.position = float4(ui_vertex.position * 2.0 - 1.0, 0.0, 1.0);
    output.color = ui_vertex.color;
    output.texture_coordinates = ui_vertex.texture_coordinates;

    return output;
}

[shader("fragment")]
float4 main(VertexOutput input) : SV_Target0 {
    if (push_constants.render_mode == RenderMode.Solid) {
        return input.color;
    }

    float4 texel = textures[push_constants.texture_index].Sample(samplers[SAMPLER_LINEAR_CLAMP], input.texture_coordinates);

    return input.color * texel.r;
}
