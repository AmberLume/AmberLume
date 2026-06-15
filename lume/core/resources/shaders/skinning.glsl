#ifndef SKINNING_GLSL
#define SKINNING_GLSL

mat4 compute_skin_matrix(Entity entity, Vertex vertex, uint64_t bone_transform_buffer_device_address) {
    if (entity.is_skinned == 0u) {
        return entity.transform_matrix;
    }

    uint bone_index_0 = vertex.bone_indices[0] & 0xFFFFu;
    uint bone_index_1 = (vertex.bone_indices[0] >> 16u) & 0xFFFFu;
    uint bone_index_2 = vertex.bone_indices[1] & 0xFFFFu;
    uint bone_index_3 = (vertex.bone_indices[1] >> 16u) & 0xFFFFu;

    BoneTransformBuffer transforms = BoneTransformBuffer(bone_transform_buffer_device_address);
    uint base = entity.bone_transform_offset;

    mat4 local_skin =
        transforms.data[base + bone_index_0].transform * vertex.bone_weights[0] +
        transforms.data[base + bone_index_1].transform * vertex.bone_weights[1] +
        transforms.data[base + bone_index_2].transform * vertex.bone_weights[2] +
        transforms.data[base + bone_index_3].transform * vertex.bone_weights[3];

    return entity.transform_matrix * local_skin;
}

mat4 compute_previous_skin_matrix(Entity entity, Vertex vertex, uint64_t bone_transform_buffer_device_address) {
    if (entity.is_skinned == 0u) {
        return entity.previous_transform_matrix;
    }

    uint bone_index_0 = vertex.bone_indices[0] & 0xFFFFu;
    uint bone_index_1 = (vertex.bone_indices[0] >> 16u) & 0xFFFFu;
    uint bone_index_2 = vertex.bone_indices[1] & 0xFFFFu;
    uint bone_index_3 = (vertex.bone_indices[1] >> 16u) & 0xFFFFu;

    BoneTransformBuffer transforms = BoneTransformBuffer(bone_transform_buffer_device_address);
    uint base = entity.bone_transform_offset;

    mat4 local_skin =
        transforms.data[base + bone_index_0].transform * vertex.bone_weights[0] +
        transforms.data[base + bone_index_1].transform * vertex.bone_weights[1] +
        transforms.data[base + bone_index_2].transform * vertex.bone_weights[2] +
        transforms.data[base + bone_index_3].transform * vertex.bone_weights[3];

    return entity.previous_transform_matrix * local_skin;
}

#endif