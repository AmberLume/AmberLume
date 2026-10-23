import "../common.slang";

struct AccelerationStructureInstance {
    float4 transform[3];

    uint instance_custom_index_and_mask;
    uint sbt_offset_and_flags;

    uint64_t acceleration_structure_reference;
};

struct PushConstants {
    Ptr<Entity, Access.Read> entity_buffer_device_address;
    Ptr<uint64_t, Access.Read> blas_address_buffer_device_address;
    Ptr<AccelerationStructureInstance> instance_buffer_device_address;
    Ptr<Mesh, Access.Read> mesh_buffer_device_address;
    Ptr<Submesh, Access.Read> submesh_buffer_device_address;
    Ptr<Material, Access.Read> material_buffer_device_address;

    uint entity_count;
};

[[vk::push_constant]] ConstantBuffer<PushConstants> push_constants;

uint instance_mask(Mesh mesh) {
    uint alpha_bits = 0u;
    for (uint local_index = 0u; local_index < mesh.submesh_count; local_index++) {
        Submesh submesh = push_constants.submesh_buffer_device_address[mesh.submesh_offset + local_index];
        alpha_bits |= push_constants.material_buffer_device_address[submesh.material_index].flags & MATERIAL_ALPHA_MODE_BITS;
    }

    uint mask = 0u;
    if ((alpha_bits & (MATERIAL_FLAG_ALPHA_OPAQUE | MATERIAL_FLAG_ALPHA_MASK)) != 0u) {
        mask |= RT_INSTANCE_MASK_OPAQUE;
    }
    if ((alpha_bits & MATERIAL_FLAG_ALPHA_BLEND) != 0u) {
        mask |= RT_INSTANCE_MASK_BLEND;
    }

    return mask;
}

[shader("compute")]
[numthreads(256, 1, 1)]
void main(uint3 global_invocation_id : SV_DispatchThreadID) {
    uint entity_index = global_invocation_id.x;
    if (entity_index >= push_constants.entity_count) {
        return;
    }

    Entity entity = push_constants.entity_buffer_device_address[entity_index];
    uint64_t blas_address = push_constants.blas_address_buffer_device_address[entity_index];

    uint mask = 0u;
    if (blas_address != 0) {
        mask = instance_mask(push_constants.mesh_buffer_device_address[entity.mesh_index]);
    }

    AccelerationStructureInstance instance;
    instance.transform[0] = entity.transform_matrix[0];
    instance.transform[1] = entity.transform_matrix[1];
    instance.transform[2] = entity.transform_matrix[2];
    instance.instance_custom_index_and_mask = (mask << 24) | (entity_index & 0xFFFFFFu);
    instance.sbt_offset_and_flags = 0u;
    instance.acceleration_structure_reference = blas_address;

    push_constants.instance_buffer_device_address[entity_index] = instance;
}
