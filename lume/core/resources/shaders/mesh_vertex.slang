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

float3 vertex_position(Ptr<VertexPosition, Access.Read> positions, uint index) {
    return float3(
        positions[index].position[0],
        positions[index].position[1],
        positions[index].position[2]
    );
}

float3 vertex_normal(Ptr<VertexNormalTangent, Access.Read> normal_tangents, uint index) {
    return float3(
        normal_tangents[index].normal[0],
        normal_tangents[index].normal[1],
        normal_tangents[index].normal[2]
    );
}

float4 vertex_tangent(Ptr<VertexNormalTangent, Access.Read> normal_tangents, uint index) {
    return float4(
        normal_tangents[index].tangent[0],
        normal_tangents[index].tangent[1],
        normal_tangents[index].tangent[2],
        normal_tangents[index].tangent[3]
    );
}

float2 vertex_uv(Ptr<VertexUv, Access.Read> uvs, uint index) {
    return float2(
        uvs[index].uv[0],
        uvs[index].uv[1]
    );
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

    float3 position = vertex_position(vertex_position_buffer, vertex_index);
    float3 normal = vertex_normal(vertex_normal_tangent_buffer, vertex_index);
    float4 tangent = vertex_tangent(vertex_normal_tangent_buffer, vertex_index);
    float2 uv = vertex_uv(vertex_uv_buffer, submesh.uv_offset + vertex_index - base_vertex);

    float4 world_position = mul(entity.transform_matrix, float4(position, 1.0));
    float4 clip_position = mul(camera_buffer.view_projection, world_position);

    float3x3 normal_matrix = (float3x3)transpose(inverse(entity.transform_matrix));
    float3 world_normal = normalize(mul(normal_matrix, normal));
    float3 world_tangent = normalize(mul(normal_matrix, tangent.xyz));
    world_tangent = normalize(world_tangent - dot(world_tangent, world_normal) * world_normal);
    float3 world_bitangent = cross(world_normal, world_tangent) * tangent.w;

    MeshSurfaceVertexOutput output;
    output.position = jitter_clip_position(clip_position, camera_buffer.jitter);
    output.tbn = float3x3(world_tangent, world_bitangent, world_normal);
    output.uv = uv;
    output.draw_id = draw_id;
    output.world_position = world_position.xyz;

    return output;
}
