import "../common.slang";

struct TerrainStitchRequest {
    uint mesh_id;
    uint edge_height_offset;
    uint level_deltas;
    uint _pad0;
};

struct PushConstants {
    Ptr<TerrainStitchRequest, Access.Read> request_buffer_device_address;
    Ptr<float, Access.Read> edge_height_buffer_device_address;
    Ptr<VertexPosition> vertex_position_buffer_device_address;
    Ptr<Mesh, Access.Read> mesh_buffer_device_address;
    Ptr<Submesh, Access.Read> submesh_buffer_device_address;

    uint node_count;
    uint nodes;
};

[[vk::push_constant]] ConstantBuffer<PushConstants> push_constants;

uint side_delta(uint level_deltas, uint side) {
    return (level_deltas >> (side * 8u)) & 0xFFu;
}

[shader("compute")]
[numthreads(256, 1, 1)]
void main(uint3 global_invocation_id : SV_DispatchThreadID) {
    uint node_index = global_invocation_id.x;
    if (node_index >= push_constants.node_count) {
        return;
    }

    uint nodes = push_constants.nodes;
    uint perimeter_nodes = nodes * 4u - 4u;

    uint request_index = node_index / perimeter_nodes;
    uint local_index = node_index % perimeter_nodes;

    TerrainStitchRequest request = push_constants.request_buffer_device_address[request_index];

    int cells = int(nodes) - 1;

    uint side;
    int column;
    int row;

    if (local_index < nodes) {
        side = 0u;
        column = 0;
        row = int(local_index);
    } else if (local_index < nodes * 2u) {
        side = 1u;
        column = cells;
        row = int(local_index - nodes);
    } else if (local_index < nodes * 3u - 2u) {
        side = 2u;
        column = 1 + int(local_index - nodes * 2u);
        row = 0;
    } else {
        side = 3u;
        column = 1 + int(local_index - (nodes * 3u - 2u));
        row = cells;
    }

    int position = side < 2u ? row : column;

    Ptr<float, Access.Read> edge_heights = push_constants.edge_height_buffer_device_address + request.edge_height_offset + side * nodes;

    float height = edge_heights[position];

    uint delta = side_delta(request.level_deltas, side);

    if (delta > 0u) {
        int step = 1 << delta;
        int lower = (position / step) * step;

        if (position != lower) {
            int upper = min(lower + step, cells);

            height = lerp(edge_heights[lower], edge_heights[upper], float(position - lower) / float(upper - lower));
        }
    }

    Mesh mesh = push_constants.mesh_buffer_device_address[request.mesh_id];
    Submesh submesh = push_constants.submesh_buffer_device_address[mesh.submesh_offset];

    uint vertex_index = submesh.vertex_offset + uint(row) * nodes + uint(column);

    push_constants.vertex_position_buffer_device_address[vertex_index].position.y = height;
}
