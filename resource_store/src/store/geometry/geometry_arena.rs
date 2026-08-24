use crate::store::geometry::mesh_regions::MeshRegions;
use anyhow::Result;
use ash::vk::{BufferUsageFlags, DeviceSize};
use gpu::BufferArray;
use gpu::ManagedBuffer;
use gpu::ManagedBufferFactory;
use gpu_allocator::MemoryLocation;
use gpu_data::MeshGPU;
use gpu_data::MeshVertexAttributeGPU;
use gpu_data::MeshVertexGPU;
use gpu_data::MeshVertexSkinGPU;
use gpu_data::SubmeshGPU;
use index_allocator::ResourceLimits;

pub struct GeometryArena {
    index_allocation: ManagedBuffer,
    mesh_allocation: ManagedBuffer,
    submesh_allocation: ManagedBuffer,
    vertex_allocation: ManagedBuffer,
    vertex_attribute_allocation: ManagedBuffer,
    vertex_skin_allocation: ManagedBuffer,

    pub mesh_regions: MeshRegions,
}

impl GeometryArena {
    pub fn create(
        buffer_factory: &ManagedBufferFactory,
        limits: &ResourceLimits,
        ray_tracing: bool,
    ) -> Result<Self> {
        let table_usage = BufferUsageFlags::STORAGE_BUFFER | BufferUsageFlags::TRANSFER_DST;

        let mut index_usage = BufferUsageFlags::INDEX_BUFFER | BufferUsageFlags::TRANSFER_DST;
        let mut vertex_usage = table_usage;

        if ray_tracing {
            index_usage |= BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR;
            vertex_usage |= BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR;
        }

        let index_allocation = Self::create_stream::<u32>(buffer_factory, "index", limits.max_indices, index_usage)?;
        let mesh_allocation = Self::create_stream::<MeshGPU>(buffer_factory, "mesh", limits.max_meshes, table_usage)?;
        let submesh_allocation = Self::create_stream::<SubmeshGPU>(buffer_factory, "submesh", limits.max_submeshes, table_usage)?;
        let vertex_allocation = Self::create_stream::<MeshVertexGPU>(buffer_factory, "mesh_vertex", limits.max_vertices, vertex_usage)?;
        let vertex_attribute_allocation = Self::create_stream::<MeshVertexAttributeGPU>(buffer_factory, "mesh_vertex_attribute", limits.max_vertex_attributes, table_usage)?;
        let vertex_skin_allocation = Self::create_stream::<MeshVertexSkinGPU>(buffer_factory, "mesh_vertex_skin", limits.max_vertex_skins, table_usage)?;

        Ok(Self {
            mesh_regions: MeshRegions {
                index: BufferArray::create(index_allocation.whole("index"), limits.max_indices),
                mesh: BufferArray::create(mesh_allocation.whole("mesh"), limits.max_meshes),
                submesh: BufferArray::create(submesh_allocation.whole("submesh"), limits.max_submeshes),
                vertex: BufferArray::create(vertex_allocation.whole("mesh_vertex"), limits.max_vertices),
                vertex_attribute: BufferArray::create(vertex_attribute_allocation.whole("mesh_vertex_attribute"), limits.max_vertex_attributes),
                vertex_skin: BufferArray::create(vertex_skin_allocation.whole("mesh_vertex_skin"), limits.max_vertex_skins),
            },

            index_allocation,
            mesh_allocation,
            submesh_allocation,
            vertex_allocation,
            vertex_attribute_allocation,
            vertex_skin_allocation,
        })
    }

    pub fn destroy(self, buffer_factory: &ManagedBufferFactory) -> Result<()> {
        buffer_factory.destroy_buffer(self.index_allocation)?;
        buffer_factory.destroy_buffer(self.mesh_allocation)?;
        buffer_factory.destroy_buffer(self.submesh_allocation)?;
        buffer_factory.destroy_buffer(self.vertex_allocation)?;
        buffer_factory.destroy_buffer(self.vertex_attribute_allocation)?;
        buffer_factory.destroy_buffer(self.vertex_skin_allocation)?;

        Ok(())
    }

    fn create_stream<T>(
        buffer_factory: &ManagedBufferFactory,
        name: &'static str,
        capacity: u32,
        usage: BufferUsageFlags,
    ) -> Result<ManagedBuffer> {
        buffer_factory.create_managed_buffer(
            name,
            capacity as DeviceSize * size_of::<T>() as DeviceSize,
            usage,
            MemoryLocation::GpuOnly,
        )
    }
}
