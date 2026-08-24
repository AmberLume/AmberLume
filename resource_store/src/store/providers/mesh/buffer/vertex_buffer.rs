use anyhow::Result;
use ash::vk;
use gpu_allocator::MemoryLocation;
use vk::{BufferUsageFlags, DeviceSize};
use resource_data::submesh_data::ArchivedSubmeshData;
use gpu::ManagedBuffer;
use gpu::ManagedBufferFactory;
use gpu_data::VertexGPU;

pub fn vertex_from_archived(submesh_data: &ArchivedSubmeshData, index: usize) -> VertexGPU {
    let position = &submesh_data.positions[index];
    let normal = &submesh_data.normals[index];
    let tangent = &submesh_data.tangents[index];
    let uv = &submesh_data.uvs[index];
    let bone_indices = &submesh_data.bone_indices[index];
    let bone_weights = &submesh_data.bone_weights[index];

    VertexGPU::new(
        position.map(|v| v.into()),
        normal.map(|v| v.into()),
        tangent.map(|v| v.into()),
        uv.map(|v| v.into()),
        [
            (bone_indices[0].to_native() as u32) | ((bone_indices[1].to_native() as u32) << 16),
            (bone_indices[2].to_native() as u32) | ((bone_indices[3].to_native() as u32) << 16),
        ],
        bone_weights.map(|v| v.into()),
    )
}

pub fn create_vertex_buffer(
    buffer_factory: &ManagedBufferFactory,
    capacity: u32,
    ray_tracing: bool,
) -> Result<ManagedBuffer> {
    let mut usage = BufferUsageFlags::STORAGE_BUFFER | BufferUsageFlags::TRANSFER_DST;

    if ray_tracing {
        usage |= BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR;
    }

    buffer_factory.create_managed_buffer(
        "vertex",
        capacity as DeviceSize * size_of::<VertexGPU>() as DeviceSize,
        usage,
        MemoryLocation::GpuOnly,
    )
}
