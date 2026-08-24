use anyhow::Result;
use ash::vk;
use gpu_allocator::MemoryLocation;
use vk::{BufferUsageFlags, DeviceSize};
use resource_data::submesh_data::ArchivedSubmeshData;
use gpu::ManagedBuffer;
use gpu::ManagedBufferFactory;
use gpu_data::MeshVertexGPU;

pub fn mesh_vertex_from_archived(submesh_data: &ArchivedSubmeshData, index: usize) -> MeshVertexGPU {
    let position = &submesh_data.positions[index];
    let normal = &submesh_data.normals[index];

    MeshVertexGPU::new(
        position.map(|v| v.into()),
        normal.map(|v| v.into()),
    )
}

pub fn create_mesh_vertex_buffer(
    buffer_factory: &ManagedBufferFactory,
    capacity: u32,
    ray_tracing: bool,
) -> Result<ManagedBuffer> {
    let mut usage = BufferUsageFlags::STORAGE_BUFFER | BufferUsageFlags::TRANSFER_DST;

    if ray_tracing {
        usage |= BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR;
    }

    buffer_factory.create_managed_buffer(
        "mesh_vertex",
        capacity as DeviceSize * size_of::<MeshVertexGPU>() as DeviceSize,
        usage,
        MemoryLocation::GpuOnly,
    )
}
