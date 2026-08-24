use gpu::BufferArray;
use gpu_data::MeshGPU;
use gpu_data::MeshVertexAttributeGPU;
use gpu_data::MeshVertexGPU;
use gpu_data::MeshVertexSkinGPU;
use gpu_data::SubmeshGPU;

#[derive(Clone, Copy)]
pub struct MeshRegions {
    pub index: BufferArray<u32>,
    pub mesh: BufferArray<MeshGPU>,
    pub submesh: BufferArray<SubmeshGPU>,
    pub vertex: BufferArray<MeshVertexGPU>,
    pub vertex_attribute: BufferArray<MeshVertexAttributeGPU>,
    pub vertex_skin: BufferArray<MeshVertexSkinGPU>,
}
