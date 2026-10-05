use anyhow::Result;
use ash::vk::{BufferUsageFlags, DeviceSize};
use gpu::ManagedBuffer;
use gpu::ManagedBufferFactory;
use gpu::RangeAllocation;
use gpu::SingleAllocation;
use gpu_allocator::MemoryLocation;
use gpu_data::AnimationFrameGPU;
use gpu_data::AnimationGPU;
use gpu_data::MaterialGPU;
use gpu_data::MeshBindingGPU;
use gpu_data::MeshGPU;
use gpu_data::MeshVertexSkinGPU;
use gpu_data::SkeletonBoneGPU;
use gpu_data::SkeletonGPU;
use gpu_data::SubmeshBoundsGPU;
use gpu_data::SubmeshGPU;
use gpu_data::VertexNormalTangentGPU;
use gpu_data::VertexPositionGPU;
use gpu_data::VertexUvGPU;
use index_allocator::ArcUnwrapOrErr;
use index_allocator::IndexManager;
use index_allocator::RangeAllocator;
use index_allocator::ResourceLimits;
use std::sync::Arc;
use crate::store::vertex_allocation::VertexAllocation;

pub struct ResourceBuffers {
    pub index: Arc<RangeAllocation<u32>>,
    pub submesh: Arc<RangeAllocation<SubmeshGPU>>,
    pub submesh_bounds: Arc<RangeAllocation<SubmeshBoundsGPU>>,
    pub vertex: Arc<VertexAllocation>,
    pub vertex_uv: Arc<RangeAllocation<VertexUvGPU>>,
    pub mesh_vertex_skin: Arc<RangeAllocation<MeshVertexSkinGPU>>,
    pub mesh_binding: Arc<RangeAllocation<MeshBindingGPU>>,
    pub skeleton_bone: Arc<RangeAllocation<SkeletonBoneGPU>>,
    pub animation_frame: Arc<RangeAllocation<AnimationFrameGPU>>,

    pub mesh: Arc<SingleAllocation<MeshGPU>>,
    pub skeleton: Arc<SingleAllocation<SkeletonGPU>>,
    pub animation: Arc<SingleAllocation<AnimationGPU>>,
    pub material: Arc<SingleAllocation<MaterialGPU>>,
}

impl ResourceBuffers {
    pub fn create(
        buffer_factory: &ManagedBufferFactory,
        limits: &ResourceLimits,
        ray_tracing: bool,
    ) -> Result<Self> {
        let table_usage = BufferUsageFlags::STORAGE_BUFFER | BufferUsageFlags::TRANSFER_DST;

        let mut index_usage = BufferUsageFlags::INDEX_BUFFER | BufferUsageFlags::TRANSFER_DST;
        let mut position_usage = table_usage;

        if ray_tracing {
            index_usage |= BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR;
            position_usage |= BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR;
        }

        Ok(Self {
            index: Arc::new(Self::create_range(buffer_factory, "index", limits.max_indices, index_usage)?),
            submesh: Arc::new(Self::create_range(buffer_factory, "submesh", limits.max_submeshes, table_usage)?),
            submesh_bounds: Arc::new(Self::create_range(buffer_factory, "submesh_bounds", limits.max_submesh_bounds, table_usage)?),
            vertex: Arc::new(VertexAllocation::create(
                Self::create_memory::<VertexPositionGPU>(buffer_factory, "vertex_position", limits.max_vertices, position_usage)?,
                Self::create_memory::<VertexNormalTangentGPU>(buffer_factory, "vertex_normal_tangent", limits.max_vertices, table_usage)?,
                RangeAllocator::new(limits.max_vertices),
            )),
            vertex_uv: Arc::new(Self::create_range(buffer_factory, "vertex_uv", limits.max_vertex_uvs, table_usage)?),
            mesh_vertex_skin: Arc::new(Self::create_range(buffer_factory, "mesh_vertex_skin", limits.max_vertex_skins, table_usage)?),
            mesh_binding: Arc::new(Self::create_range(buffer_factory, "mesh_binding", limits.max_mesh_bindings, table_usage)?),
            skeleton_bone: Arc::new(Self::create_range(buffer_factory, "skeleton_bone", limits.max_skeleton_bones, table_usage)?),
            animation_frame: Arc::new(Self::create_range(buffer_factory, "animation_frame", limits.max_animation_frames, table_usage)?),

            mesh: Arc::new(Self::create_single(buffer_factory, "mesh", limits.max_meshes, table_usage)?),
            skeleton: Arc::new(Self::create_single(buffer_factory, "skeleton", limits.max_skeletons, table_usage)?),
            animation: Arc::new(Self::create_single(buffer_factory, "animation", limits.max_animations, table_usage)?),
            material: Arc::new(Self::create_single(buffer_factory, "material", limits.max_materials, table_usage)?),
        })
    }

    fn create_single<T>(
        buffer_factory: &ManagedBufferFactory,
        label: &'static str,
        capacity: u32,
        usage: BufferUsageFlags,
    ) -> Result<SingleAllocation<T>> {
        let allocation = buffer_factory.create_managed_buffer(
            label,
            capacity as DeviceSize * size_of::<T>() as DeviceSize,
            usage,
            MemoryLocation::GpuOnly,
        )?;

        Ok(SingleAllocation::create(
            allocation,
            Arc::new(IndexManager::new(capacity)),
        ))
    }

    fn create_range<T>(
        buffer_factory: &ManagedBufferFactory,
        label: &'static str,
        capacity: u32,
        usage: BufferUsageFlags,
    ) -> Result<RangeAllocation<T>> {
        let allocation = Self::create_memory::<T>(buffer_factory, label, capacity, usage)?;

        Ok(RangeAllocation::create(allocation, RangeAllocator::new(capacity)))
    }

    fn create_memory<T>(
        buffer_factory: &ManagedBufferFactory,
        label: &'static str,
        capacity: u32,
        usage: BufferUsageFlags,
    ) -> Result<ManagedBuffer> {
        buffer_factory.create_managed_buffer(
            label,
            capacity as DeviceSize * size_of::<T>() as DeviceSize,
            usage,
            MemoryLocation::GpuOnly,
        )
    }

    pub fn destroy(self, buffer_factory: &ManagedBufferFactory) -> Result<()> {
        buffer_factory.destroy_buffer(self.index.try_unwrap()?.allocation)?;
        buffer_factory.destroy_buffer(self.submesh.try_unwrap()?.allocation)?;
        buffer_factory.destroy_buffer(self.submesh_bounds.try_unwrap()?.allocation)?;

        let vertex = self.vertex.try_unwrap()?;
        buffer_factory.destroy_buffer(vertex.position)?;
        buffer_factory.destroy_buffer(vertex.normal_tangent)?;

        buffer_factory.destroy_buffer(self.vertex_uv.try_unwrap()?.allocation)?;
        buffer_factory.destroy_buffer(self.mesh_vertex_skin.try_unwrap()?.allocation)?;
        buffer_factory.destroy_buffer(self.mesh_binding.try_unwrap()?.allocation)?;
        buffer_factory.destroy_buffer(self.skeleton_bone.try_unwrap()?.allocation)?;
        buffer_factory.destroy_buffer(self.animation_frame.try_unwrap()?.allocation)?;

        buffer_factory.destroy_buffer(self.mesh.try_unwrap()?.allocation)?;
        buffer_factory.destroy_buffer(self.skeleton.try_unwrap()?.allocation)?;
        buffer_factory.destroy_buffer(self.animation.try_unwrap()?.allocation)?;
        buffer_factory.destroy_buffer(self.material.try_unwrap()?.allocation)?;

        Ok(())
    }
}
