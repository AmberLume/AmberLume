import "../common.slang";
import "../fullscreen.slang";
import "../math.slang";
import "../sh.slang";
import "../sky.slang";

struct PushConstants {
    Ptr<Scene, Access.Read> scene_buffer_device_address;
};

[[vk::push_constant]] ConstantBuffer<PushConstants> push_constants;

static const uint SAMPLE_COUNT = 64u;

float3 fibonacci_direction(uint i, uint n) {
    float k = float(i) + 0.5;
    float cos_phi = 1.0 - 2.0 * k / float(n);
    float sin_phi = sqrt(max(1.0 - cos_phi * cos_phi, 0.0));
    float theta = PI * (1.0 + sqrt(5.0)) * k;

    return float3(sin_phi * cos(theta), sin_phi * sin(theta), cos_phi);
}

[shader("vertex")]
float4 main(uint vertex_index : SV_VulkanVertexID) : SV_Position {
    return fullscreen_position(vertex_index);
}

[shader("fragment")]
float4 main(float4 position : SV_Position) : SV_Target0 {
    uint coefficient = uint(position.x);
    Ptr<Scene, Access.Read> scene = push_constants.scene_buffer_device_address;
    float3 sun_direction = normalize(-scene.light_direction);

    float solid_angle = 4.0 * PI / float(SAMPLE_COUNT);

    float3 accumulated = float3(0.0);
    for (uint i = 0u; i < SAMPLE_COUNT; i++) {
        float3 direction = fibonacci_direction(i, SAMPLE_COUNT);
        float3 radiance = procedural_sky(direction, sun_direction, scene.time);

        float y[9];
        sh_basis(direction, y);

        accumulated += radiance * y[coefficient] * solid_angle;
    }

    return float4(accumulated, 1.0);
}
