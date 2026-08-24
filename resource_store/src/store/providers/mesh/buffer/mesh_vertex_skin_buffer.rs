use gpu_data::MeshVertexSkinGPU;
use resource_data::submesh_data::ArchivedSubmeshData;

pub fn mesh_vertex_skin_from_archived(
    submesh_data: &ArchivedSubmeshData,
    index: usize,
) -> MeshVertexSkinGPU {
    let bone_indices = &submesh_data.bone_indices[index];
    let bone_weights = &submesh_data.bone_weights[index];

    MeshVertexSkinGPU::new(
        [
            (bone_indices[0].to_native() as u32) | ((bone_indices[1].to_native() as u32) << 16),
            (bone_indices[2].to_native() as u32) | ((bone_indices[3].to_native() as u32) << 16),
        ],
        bone_weights.map(|v| v.into()),
    )
}
