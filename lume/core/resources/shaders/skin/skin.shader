import "../common.slang";
import "../math.slang";

struct SkinTarget {
    uint source_vertex_offset;
    uint source_skin_offset;
    uint target_vertex_offset;
    uint skinning_instance_index;
    uint first_vertex;
};

struct MeshVertexSkin {
    uint2 bone_indices;
    float4 bone_weights;
};

struct PushConstants {
    Ptr<SkinTarget, Access.Read> skin_target_buffer_device_address;
    Ptr<SkinningInstance, Access.Read> skinning_instance_buffer_device_address;
    Ptr<BoneTransform, Access.Read> bone_transform_buffer_device_address;
    Ptr<VertexPosition> vertex_position_buffer_device_address;
    Ptr<VertexNormalTangent> vertex_normal_tangent_buffer_device_address;
    Ptr<MeshVertexSkin, Access.Read> vertex_skin_buffer_device_address;

    uint target_count;
    uint vertex_count;
};

[[vk::push_constant]] ConstantBuffer<PushConstants> push_constants;

uint find_target(uint vertex) {
    uint first = 0u;
    uint last = push_constants.target_count;

    while (last - first > 1u) {
        uint middle = (first + last) / 2u;

        if (push_constants.skin_target_buffer_device_address[middle].first_vertex <= vertex) {
            first = middle;
        } else {
            last = middle;
        }
    }

    return first;
}

float4x4 skin_matrix(MeshVertexSkin skin, uint bone_transform_offset) {
    Ptr<BoneTransform, Access.Read> bone_transforms = push_constants.bone_transform_buffer_device_address + bone_transform_offset;

    uint bone_index_0 = skin.bone_indices.x & 0xFFFFu;
    uint bone_index_1 = (skin.bone_indices.x >> 16u) & 0xFFFFu;
    uint bone_index_2 = skin.bone_indices.y & 0xFFFFu;
    uint bone_index_3 = (skin.bone_indices.y >> 16u) & 0xFFFFu;

    return bone_transforms[bone_index_0].transform * skin.bone_weights.x
        + bone_transforms[bone_index_1].transform * skin.bone_weights.y
        + bone_transforms[bone_index_2].transform * skin.bone_weights.z
        + bone_transforms[bone_index_3].transform * skin.bone_weights.w;
}

[shader("compute")]
[numthreads(256, 1, 1)]
void main(uint3 global_invocation_id : SV_DispatchThreadID) {
    uint vertex = global_invocation_id.x;
    if (vertex >= push_constants.vertex_count) {
        return;
    }

    Ptr<VertexPosition, Access.Read> source_positions = (Ptr<VertexPosition, Access.Read>)push_constants.vertex_position_buffer_device_address;
    Ptr<VertexNormalTangent, Access.Read> source_normal_tangents = (Ptr<VertexNormalTangent, Access.Read>)push_constants.vertex_normal_tangent_buffer_device_address;

    SkinTarget target = push_constants.skin_target_buffer_device_address[find_target(vertex)];
    SkinningInstance instance = push_constants.skinning_instance_buffer_device_address[target.skinning_instance_index];

    uint local_vertex = vertex - target.first_vertex;
    uint source_vertex = target.source_vertex_offset + local_vertex;
    uint target_vertex = target.target_vertex_offset + local_vertex;

    float4x4 skin_transform = skin_matrix(push_constants.vertex_skin_buffer_device_address[target.source_skin_offset + local_vertex], instance.bone_transform_offset);

    float3 position = source_positions[source_vertex].position;
    VertexNormalTangent normal_tangent = source_normal_tangents[source_vertex];

    push_constants.vertex_position_buffer_device_address[target_vertex] = { mul(skin_transform, float4(position, 1.0)).xyz };
    push_constants.vertex_normal_tangent_buffer_device_address[target_vertex] = {
        normalize(mul(normal_matrix(skin_transform), normal_tangent.normal)),
        float4(normalize(mul((float3x3)skin_transform, normal_tangent.tangent.xyz)), normal_tangent.tangent.w)
    };
}
