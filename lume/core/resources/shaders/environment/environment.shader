import "../common.slang";
import "../fullscreen.slang";
import "../projection.slang";
import "../sky.slang";

struct PushConstants {
    Ptr<Scene, Access.Read> scene_buffer_device_address;
    Ptr<Camera, Access.Read> camera_buffer_device_address;
};

[[vk::push_constant]] ConstantBuffer<PushConstants> push_constants;

struct FragmentOutput {
    float4 color : SV_Target0;
    float2 velocity : SV_Target1;
};

[shader("vertex")]
FullscreenVertexOutput main(uint vertex_index : SV_VulkanVertexID) {
    return fullscreen_vertex(vertex_index);
}

[shader("fragment")]
FragmentOutput main(FullscreenVertexOutput input) {
    Ptr<Scene, Access.Read> scene = push_constants.scene_buffer_device_address;
    Ptr<Camera, Access.Read> camera = push_constants.camera_buffer_device_address;

    float2 ndc = input.uv * 2.0 - 1.0;

    float3 direction = view_ray_direction(camera.inverse_view_projection, ndc);
    float3 sun_direction = normalize(-scene.light_direction);

    float4 previous_clip = mul(camera.previous_view_projection, float4(direction, 0.0));

    FragmentOutput output;
    output.color = float4(procedural_sky(direction, sun_direction, scene.time) + procedural_sun(direction, sun_direction), 1.0);
    output.velocity = previous_clip.w > 0.0 ? (previous_clip.xy / previous_clip.w - ndc) * 0.5 : float2(2.0);

    return output;
}
