use gpu_data::MeshVertexSkinGPU;
use gpu_data::VertexNormalTangentGPU;
use gpu_data::VertexPositionGPU;
use gpu_data::VertexUvGPU;
use resource_data::submesh_data::ArchivedSubmeshData;
use resource_residency::ResRef;
use std::sync::Arc;

#[derive(Clone)]
pub struct SubmeshConfig {
    pub indices: Vec<u32>,

    pub positions: Vec<VertexPositionGPU>,
    pub normal_tangents: Vec<VertexNormalTangentGPU>,
    pub uvs: Vec<VertexUvGPU>,
    pub skins: Vec<MeshVertexSkinGPU>,

    pub material: Arc<ResRef>,
    pub bounds: [f32; 6],
}

impl SubmeshConfig {
    pub(crate) fn from_archived(
        submesh_data: &ArchivedSubmeshData,
        skinned: bool,
        material: Arc<ResRef>,
    ) -> Self {
        let vertex_count = submesh_data.positions.len();

        let indices = submesh_data.indices.iter()
            .map(|v| v.to_native())
            .collect::<Vec<_>>();
        let positions = (0..vertex_count).map(|index| {
            VertexPositionGPU::new(submesh_data.positions[index].map(|v| v.into()))
        }).collect::<Vec<_>>();
        let normal_tangents = (0..vertex_count).map(|index| {
            VertexNormalTangentGPU::new(
                submesh_data.normals[index].map(|v| v.into()),
                submesh_data.tangents[index].map(|v| v.into()),
            )
        }).collect::<Vec<_>>();
        let uvs = (0..vertex_count).map(|index| {
            VertexUvGPU::new(submesh_data.uvs[index].map(|v| v.into()))
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

            positions,
            normal_tangents,
            uvs,
            skins,

            material,
            bounds: submesh_data.bounds.map(|v| v.into()),
        }
    }
}
