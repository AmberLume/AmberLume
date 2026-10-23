import "../common.slang";
import "../bindings.slang";
import "../fullscreen.slang";
import "../projection.slang";
import "../sh.slang";

enum DebugLayerKind : uint {
    Velocity = 1,
    Normal = 2,
    Gtao = 3,
    ShIrradiance = 4,
    HizNear = 5,
    HizFar = 6,
    Shadow = 7,
    AoHistory = 8,
    AoDenoised = 9,
};

struct PushConstants {
    Ptr<Camera, Access.Read> camera_buffer_device_address;

    uint texture_descriptor_id;
    DebugLayerKind layer_kind;
    uint shadow_colored;

    float denoise_history;
};

[[vk::push_constant]] ConstantBuffer<PushConstants> push_constants;

[shader("vertex")]
FullscreenVertexOutput main(uint vertex_index : SV_VulkanVertexID) {
    return fullscreen_vertex(vertex_index);
}

[shader("fragment")]
float4 main(FullscreenVertexOutput input) : SV_Target0 {
    uint texture_descriptor_id = push_constants.texture_descriptor_id;

    if (push_constants.layer_kind == DebugLayerKind.ShIrradiance) {
        float3 direction = view_ray_direction(push_constants.camera_buffer_device_address.inverse_view_projection, input.uv * 2.0 - 1.0);

        return float4(sh_texture_irradiance(texture_descriptor_id, direction), 1.0);
    }

    float4 sampled = graph_textures[texture_descriptor_id].Sample(samplers[SAMPLER_LINEAR_CLAMP], input.uv);

    float3 color;
    switch (push_constants.layer_kind) {
        case DebugLayerKind.Velocity:
            color = float3(0.5 + sampled.rg * 20.0, 0.5);
            break;
        case DebugLayerKind.Normal:
            color = sampled.rgb * 0.5 + 0.5;
            break;
        case DebugLayerKind.HizFar:
            color = sampled.ggg;
            break;
        case DebugLayerKind.AoHistory:
            color = lerp(float3(1.0, 0.0, 0.0), float3(0.0, 1.0, 0.0), clamp(sampled.g / max(push_constants.denoise_history, 1.0), 0.0, 1.0));
            break;
        case DebugLayerKind.Shadow:
            color = push_constants.shadow_colored == 1u ? sampled.rgb : sampled.rrr;
            break;
        default:
            color = sampled.rrr;
            break;
    }

    return float4(color, 1.0);
}
