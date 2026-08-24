use anyhow::Result;
use ash::vk;
use gpu_allocator::MemoryLocation;
use vk::{BufferUsageFlags, DeviceSize};
use resource_data::submesh_data::ArchivedSubmeshData;
use gpu::ManagedBuffer;
use gpu::ManagedBufferFactory;
use gpu_data::MeshVertexAttributeGPU;

pub fn mesh_vertex_attribute_from_archived(submesh_data: &ArchivedSubmeshData, index: usize) -> MeshVertexAttributeGPU {
    let tangent = &submesh_data.tangents[index];
    let uv = &submesh_data.uvs[index];

    MeshVertexAttributeGPU::new(
        tangent.map(|v| v.into()),
        uv.map(|v| v.into()),
    )
}

pub fn create_mesh_vertex_attribute_buffer(
    buffer_factory: &ManagedBufferFactory,
    capacity: u32,
) -> Result<ManagedBuffer> {
    buffer_factory.create_managed_buffer(
        "mesh_vertex_attribute",
        capacity as DeviceSize * size_of::<MeshVertexAttributeGPU>() as DeviceSize,
        BufferUsageFlags::STORAGE_BUFFER | BufferUsageFlags::TRANSFER_DST,
        MemoryLocation::GpuOnly,
    )
}
