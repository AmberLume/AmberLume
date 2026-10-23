import "common.slang";
import "math.slang";
import "projection.slang";

struct MeshSurfaceVertexOutput {
    float4 position : SV_Position;
    float3x3 tbn : TBN;
    float2 uv : UV;
    nointerpolation uint draw_id : DRAW_ID;
    float3 world_position : WORLD_POSITION;
};

struct MeshClipPositions {
    float4 current;
    float4 previous;
};

MeshClipPositions mesh_clip_positions(
    Ptr<Camera, Access.Read> camera,
    Ptr<VertexPosition, Access.Read> vertex_position_buffer,
    Entity entity,
    DrawData draw_data,
    uint vertex_index
) {
    float3 position = vertex_position_buffer[vertex_index].position;
    float3 previous_position = vertex_position_buffer[uint(int(vertex_index) + draw_data.previous_vertex_delta)].position;

    MeshClipPositions clip;
    clip.current = mul(camera.view_projection, mul(entity.transform_matrix, float4(position, 1.0)));
    clip.previous = mul(camera.previous_view_projection, mul(entity.previous_transform_matrix, float4(previous_position, 1.0)));

    return clip;
}

MeshSurfaceVertexOutput mesh_surface_vertex(
    Ptr<Camera, Access.Read> camera_buffer,
    Ptr<DrawData, Access.Read> draw_data_buffer,
    Ptr<Entity, Access.Read> entity_buffer,
    Ptr<Submesh, Access.Read> submesh_buffer,
    Ptr<VertexPosition, Access.Read> vertex_position_buffer,
    Ptr<VertexNormalTangent, Access.Read> vertex_normal_tangent_buffer,
    Ptr<VertexUv, Access.Read> vertex_uv_buffer,
    uint vertex_index,
    uint base_vertex,
    uint draw_id
) {
    DrawData draw_data = draw_data_buffer[draw_id];
    Entity entity = entity_buffer[draw_data.entity_index];
    Submesh submesh = submesh_buffer[draw_data.submesh_index];

    float3 position = vertex_position_buffer[vertex_index].position;
    VertexNormalTangent normal_tangent = vertex_normal_tangent_buffer[vertex_index];
    float2 uv = vertex_uv_buffer[submesh.uv_offset + vertex_index - base_vertex].uv;

    float4 world_position = mul(entity.transform_matrix, float4(position, 1.0));
    float4 clip_position = mul(camera_buffer.view_projection, world_position);

    float3 world_normal = normalize(mul(normal_matrix(entity.transform_matrix), normal_tangent.normal));
    float3 world_tangent = normalize(mul((float3x3)entity.transform_matrix, normal_tangent.tangent.xyz));
    world_tangent = normalize(world_tangent - dot(world_tangent, world_normal) * world_normal);
    float3 world_bitangent = cross(world_normal, world_tangent) * normal_tangent.tangent.w;

    MeshSurfaceVertexOutput output;
    output.position = jitter_clip_position(clip_position, camera_buffer.jitter);
    output.tbn = float3x3(world_tangent, world_bitangent, world_normal);
    output.uv = uv;
    output.draw_id = draw_id;
    output.world_position = world_position.xyz;

    return output;
}
