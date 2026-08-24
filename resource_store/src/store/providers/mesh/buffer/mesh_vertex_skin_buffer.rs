use anyhow::Result;
use ash::vk;
use gpu_allocator::MemoryLocation;
use vk::{BufferUsageFlags, DeviceSize};
use resource_data::submesh_data::ArchivedSubmeshData;
use gpu::ManagedBuffer;
use gpu::ManagedBufferFactory;
use gpu_data::MeshVertexSkinGPU;

pub fn mesh_vertex_skin_from_archived(submesh_data: &ArchivedSubmeshData, index: usize) -> MeshVertexSkinGPU {
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

pub fn create_mesh_vertex_skin_buffer(
    buffer_factory: &ManagedBufferFactory,
    capacity: u32,
) -> Result<ManagedBuffer> {
    buffer_factory.create_managed_buffer(
        "mesh_vertex_skin",
        capacity as DeviceSize * size_of::<MeshVertexSkinGPU>() as DeviceSize,
        BufferUsageFlags::STORAGE_BUFFER | BufferUsageFlags::TRANSFER_DST,
        MemoryLocation::GpuOnly,
    )
}
