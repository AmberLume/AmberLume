use gpu_data::MeshVertexAttributeGPU;
use gpu_data::MeshVertexGPU;
use gpu_data::MeshVertexSkinGPU;
use resource_data::submesh_data::ArchivedSubmeshData;
use resource_residency::ResRef;
use std::sync::Arc;

pub(crate) struct ExtractedSubmesh {
    pub indices: Vec<u32>,

    pub vertices: Vec<MeshVertexGPU>,
    pub attributes: Vec<MeshVertexAttributeGPU>,
    pub skins: Vec<MeshVertexSkinGPU>,

    pub material: Arc<ResRef>,
    pub bounds: [f32; 6],
}

impl ExtractedSubmesh {
    pub fn from_archived(
        submesh_data: &ArchivedSubmeshData,
        skinned: bool,
        material: Arc<ResRef>,
    ) -> Self {
        let vertex_count = submesh_data.positions.len();

        let indices = submesh_data.indices.iter()
            .map(|v| v.to_native())
            .collect::<Vec<_>>();
        let vertices = (0..vertex_count).map(|index| {
            MeshVertexGPU::new(
                submesh_data.positions[index].map(|v| v.into()),
                submesh_data.normals[index].map(|v| v.into()),
            )
        }).collect::<Vec<_>>();
        let attributes = (0..vertex_count).map(|index| {
            MeshVertexAttributeGPU::new(
                submesh_data.tangents[index].map(|v| v.into()),
                submesh_data.uvs[index].map(|v| v.into()),
            )
        }).collect::<Vec<_>>();
        let skins = if skinned {
            (0..vertex_count).map(|index| {
                let bone_indices = &submesh_data.bone_indices[index];

                MeshVertexSkinGPU::new(
                    [
                        (bone_indices[0].to_native() as u32) | ((bone_indices[1].to_native() as u32) << 16),
                        (bone_indices[2].to_native() as u32) | ((bone_indices[3].to_native() as u32) << 16),
                    ],
                    submesh_data.bone_weights[index].map(|v| v.into()),
                )
            }).collect::<Vec<_>>()
        } else {
            Vec::new()
        };

        Self {
            indices,

            vertices,
            attributes,
            skins,

            material,
            bounds: submesh_data.bounds.map(|v| v.into()),
        }
    }
}
