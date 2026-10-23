import "../common.slang";

struct TerrainChunkView {
    float3 center;
    uint level;
    uint mesh_id;

    uint _pad0[3];
};

struct PushConstants {
    Ptr<Camera, Access.Read> camera_buffer_device_address;
    Ptr<TerrainChunkView, Access.Read> chunk_buffer_device_address;
    Ptr<VertexPosition, Access.Read> vertex_position_buffer_device_address;
    Ptr<Mesh, Access.Read> mesh_buffer_device_address;
    Ptr<Submesh, Access.Read> submesh_buffer_device_address;

    uint node_count;
    float point_size;
    float viewport_height;

    uint _pad0;
};

[[vk::push_constant]] ConstantBuffer<PushConstants> push_constants;

struct VertexOutput {
    float4 position : SV_Position;
    float point_size : SV_PointSize;
    float4 color : COLOR;
};

float3 level_color(uint level) {
    const float intensity = 1.6;

    switch (level) {
        case 0u: return float3(1.0, 0.15, 0.15) * intensity;
        case 1u: return float3(1.0, 0.55, 0.05) * intensity;
        case 2u: return float3(1.0, 1.0, 0.15) * intensity;
        case 3u: return float3(0.2, 1.0, 0.25) * intensity;
        case 4u: return float3(0.2, 0.55, 1.0) * intensity;
        default: return float3(0.7, 0.25, 1.0) * intensity;
    }
}

[shader("vertex")]
VertexOutput main(uint vertex_index : SV_VulkanVertexID) {
    Ptr<Camera, Access.Read> camera = push_constants.camera_buffer_device_address;

    uint chunk_index = vertex_index / push_constants.node_count;
    uint local_index = vertex_index % push_constants.node_count;

    TerrainChunkView chunk = push_constants.chunk_buffer_device_address[chunk_index];

    Mesh mesh = push_constants.mesh_buffer_device_address[chunk.mesh_id];
    Submesh submesh = push_constants.submesh_buffer_device_address[mesh.submesh_offset];

    float3 world = chunk.center + push_constants.vertex_position_buffer_device_address[submesh.vertex_offset + local_index].position;

    float3 camera_up = camera.view[1].xyz;

    float4 clip = mul(camera.view_projection, float4(world, 1.0));
    float4 clip_offset = mul(camera.view_projection, float4(world + camera_up * push_constants.point_size, 1.0));

    float projected = distance(clip.xy / clip.w, clip_offset.xy / clip_offset.w);

    VertexOutput output;
    output.position = clip;
    output.point_size = clamp(projected * push_constants.viewport_height * 0.5, 1.0, 64.0);
    output.color = float4(level_color(chunk.level), 1.0);

    return output;
}

[shader("fragment")]
float4 main(float4 color : COLOR) : SV_Target0 {
    return color;
}
