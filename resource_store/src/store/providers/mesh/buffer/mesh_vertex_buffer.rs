use gpu_data::MeshVertexGPU;
use resource_data::submesh_data::ArchivedSubmeshData;

pub fn mesh_vertex_from_archived(
    submesh_data: &ArchivedSubmeshData,
    index: usize,
) -> MeshVertexGPU {
    let position = &submesh_data.positions[index];
    let normal = &submesh_data.normals[index];

    MeshVertexGPU::new(position.map(|v| v.into()), normal.map(|v| v.into()))
}
