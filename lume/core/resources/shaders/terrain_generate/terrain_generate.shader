import "../common.slang";

struct TerrainGenerateRequest {
    uint mesh_id;
    uint height_offset;
    float cell_size;
    uint _pad0;
};

struct PushConstants {
    Ptr<TerrainGenerateRequest, Access.Read> request_buffer_device_address;
    Ptr<float, Access.Read> height_buffer_device_address;
    Ptr<VertexPosition> vertex_position_buffer_device_address;
    Ptr<VertexNormalTangent> vertex_normal_tangent_buffer_device_address;
    Ptr<Mesh, Access.Read> mesh_buffer_device_address;
    Ptr<Submesh, Access.Read> submesh_buffer_device_address;

    uint node_count;
    uint nodes;
    uint window_stride;
};

[[vk::push_constant]] ConstantBuffer<PushConstants> push_constants;

float window_height(uint height_offset, int column, int row) {
    int border = int(push_constants.window_stride - push_constants.nodes) / 2;
    int stride = int(push_constants.window_stride);

    return push_constants.height_buffer_device_address[height_offset + uint((row + border) * stride + (column + border))];
}

[shader("compute")]
[numthreads(256, 1, 1)]
void main(uint3 global_invocation_id : SV_DispatchThreadID) {
    uint node_index = global_invocation_id.x;
    if (node_index >= push_constants.node_count) {
        return;
    }

    uint nodes = push_constants.nodes;
    uint chunk_nodes = nodes * nodes;

    uint request_index = node_index / chunk_nodes;
    uint local_index = node_index % chunk_nodes;

    TerrainGenerateRequest request = push_constants.request_buffer_device_address[request_index];

    int column = int(local_index % nodes);
    int row = int(local_index / nodes);

    float height = window_height(request.height_offset, column, row);
    float left = window_height(request.height_offset, column - 1, row);
    float right = window_height(request.height_offset, column + 1, row);
    float back = window_height(request.height_offset, column, row - 1);
    float front = window_height(request.height_offset, column, row + 1);

    float cell_size = request.cell_size;
    float half_size = float(nodes - 1u) * cell_size * 0.5;
    float span = 2.0 * cell_size;

    float3 position = float3(
        float(column) * cell_size - half_size,
        height,
        float(row) * cell_size - half_size
    );

    float3 normal = normalize(float3(left - right, span, back - front));
    float3 tangent = normalize(float3(span, right - left, 0.0));

    Mesh mesh = push_constants.mesh_buffer_device_address[request.mesh_id];
    Submesh submesh = push_constants.submesh_buffer_device_address[mesh.submesh_offset];

    uint vertex_index = submesh.vertex_offset + local_index;

    push_constants.vertex_position_buffer_device_address[vertex_index] = { position };
    push_constants.vertex_normal_tangent_buffer_device_address[vertex_index] = { normal, float4(tangent, 1.0) };
}
