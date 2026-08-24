use gpu_data::MeshVertexAttributeGPU;
use resource_data::submesh_data::ArchivedSubmeshData;

pub fn mesh_vertex_attribute_from_archived(
    submesh_data: &ArchivedSubmeshData,
    index: usize,
) -> MeshVertexAttributeGPU {
    let tangent = &submesh_data.tangents[index];
    let uv = &submesh_data.uvs[index];

    MeshVertexAttributeGPU::new(tangent.map(|v| v.into()), uv.map(|v| v.into()))
}
