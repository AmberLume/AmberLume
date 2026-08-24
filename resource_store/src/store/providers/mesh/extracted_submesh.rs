use gpu_data::MeshVertexAttributeGPU;
use gpu_data::MeshVertexGPU;
use gpu_data::MeshVertexSkinGPU;
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
