#ifndef SKINNING_GLSL
#define SKINNING_GLSL

#include "common.glsl"

mat4 compute_local_skin_matrix(
    uint skin_index,
    uint bone_transform_offset,
    uint64_t mesh_vertex_skin_buffer_device_address,
    uint64_t bone_transform_buffer_device_address
) {
    MeshVertexSkin skin = MeshVertexSkinBuffer(mesh_vertex_skin_buffer_device_address).data[skin_index];

    uint bone_index_0 = skin.bone_indices[0] & 0xFFFFu;
    uint bone_index_1 = (skin.bone_indices[0] >> 16u) & 0xFFFFu;
    uint bone_index_2 = skin.bone_indices[1] & 0xFFFFu;
    uint bone_index_3 = (skin.bone_indices[1] >> 16u) & 0xFFFFu;

    BoneTransformBuffer transforms = BoneTransformBuffer(bone_transform_buffer_device_address);

    return
        transforms.data[bone_transform_offset + bone_index_0].transform * skin.bone_weights[0] +
        transforms.data[bone_transform_offset + bone_index_1].transform * skin.bone_weights[1] +
        transforms.data[bone_transform_offset + bone_index_2].transform * skin.bone_weights[2] +
        transforms.data[bone_transform_offset + bone_index_3].transform * skin.bone_weights[3];
}

mat4 compute_skin_matrix(
    mat4 transform_matrix,
    uint bone_transform_offset,
    uint skin_index,
    uint64_t mesh_vertex_skin_buffer_device_address,
    uint64_t bone_transform_buffer_device_address
) {
    if (bone_transform_offset == BONE_TRANSFORM_NONE) {
        return transform_matrix;
    }

    return transform_matrix * compute_local_skin_matrix(
        skin_index,
        bone_transform_offset,
        mesh_vertex_skin_buffer_device_address,
        bone_transform_buffer_device_address
    );
}

#endif
