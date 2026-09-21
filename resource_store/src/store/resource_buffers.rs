use anyhow::Result;
use ash::vk::{BufferUsageFlags, DeviceSize};
use gpu::ManagedBufferFactory;
use gpu::RangeAllocation;
use gpu::SingleAllocation;
use gpu_allocator::MemoryLocation;
use gpu_data::AnimationFrameGPU;
use gpu_data::AnimationGPU;
use gpu_data::MaterialGPU;
use gpu_data::MeshBoneGPU;
use gpu_data::MeshGPU;
use gpu_data::MeshVertexAttributeGPU;
use gpu_data::MeshVertexGPU;
use gpu_data::MeshVertexSkinGPU;
use gpu_data::SkeletonBoneGPU;
use gpu_data::SkeletonGPU;
use gpu_data::SubmeshGPU;
use index_allocator::ArcUnwrapOrErr;
use index_allocator::IndexManager;
use index_allocator::RangeAllocator;
use index_allocator::ResourceLimits;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;

pub struct ResourceBuffers {
    pub index: Arc<RangeAllocation<u32>>,
    pub submesh: Arc<RangeAllocation<SubmeshGPU>>,
    pub mesh_vertex: Arc<RangeAllocation<MeshVertexGPU>>,
    pub mesh_vertex_attribute: Arc<RangeAllocation<MeshVertexAttributeGPU>>,
    pub mesh_vertex_skin: Arc<RangeAllocation<MeshVertexSkinGPU>>,
    pub mesh_bone: Arc<RangeAllocation<MeshBoneGPU>>,
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
        frames_in_flight: u32,
        frame_counter: Arc<AtomicU64>,
    ) -> Result<Self> {
        let table_usage = BufferUsageFlags::STORAGE_BUFFER | BufferUsageFlags::TRANSFER_DST;

        let mut index_usage = BufferUsageFlags::INDEX_BUFFER | BufferUsageFlags::TRANSFER_DST;
        let mut vertex_usage = table_usage;

        if ray_tracing {
            index_usage |= BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR;
            vertex_usage |= BufferUsageFlags::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY_KHR;
        }

        Ok(Self {
            index: Arc::new(Self::create_range(buffer_factory, "index", limits.max_indices, index_usage)?),
            submesh: Arc::new(Self::create_range(buffer_factory, "submesh", limits.max_submeshes, table_usage)?),
            mesh_vertex: Arc::new(Self::create_range(buffer_factory, "mesh_vertex", limits.max_vertices, vertex_usage)?),
            mesh_vertex_attribute: Arc::new(Self::create_range(buffer_factory, "mesh_vertex_attribute", limits.max_vertex_attributes, table_usage)?),
            mesh_vertex_skin: Arc::new(Self::create_range(buffer_factory, "mesh_vertex_skin", limits.max_vertex_skins, table_usage)?),
            mesh_bone: Arc::new(Self::create_range(buffer_factory, "mesh_bone", limits.max_mesh_bones, table_usage)?),
            skeleton_bone: Arc::new(Self::create_range(buffer_factory, "skeleton_bone", limits.max_skeleton_bones, table_usage)?),
            animation_frame: Arc::new(Self::create_range(buffer_factory, "animation_frame", limits.max_animation_frames, table_usage)?),

            mesh: Arc::new(Self::create_single(buffer_factory, "mesh", limits.max_meshes, table_usage, frames_in_flight, frame_counter.clone())?),
            skeleton: Arc::new(Self::create_single(buffer_factory, "skeleton", limits.max_skeletons, table_usage, frames_in_flight, frame_counter.clone())?),
            animation: Arc::new(Self::create_single(buffer_factory, "animation", limits.max_animations, table_usage, frames_in_flight, frame_counter.clone())?),
            material: Arc::new(Self::create_single(buffer_factory, "materials", limits.max_materials, table_usage, frames_in_flight, frame_counter)?),
        })
    }

    fn create_single<T>(
        buffer_factory: &ManagedBufferFactory,
        name: &'static str,
        capacity: u32,
        usage: BufferUsageFlags,
        frames_in_flight: u32,
        frame_counter: Arc<AtomicU64>,
    ) -> Result<SingleAllocation<T>> {
        let allocation = buffer_factory.create_managed_buffer(
            name,
            capacity as DeviceSize * size_of::<T>() as DeviceSize,
            usage,
            MemoryLocation::GpuOnly,
        )?;

        Ok(SingleAllocation::create(
            allocation,
            Arc::new(IndexManager::new(capacity, frames_in_flight, frame_counter)),
        ))
    }

    fn create_range<T>(
        buffer_factory: &ManagedBufferFactory,
        name: &'static str,
        capacity: u32,
        usage: BufferUsageFlags,
    ) -> Result<RangeAllocation<T>> {
        let allocation = buffer_factory.create_managed_buffer(
            name,
            capacity as DeviceSize * size_of::<T>() as DeviceSize,
            usage,
            MemoryLocation::GpuOnly,
        )?;

        Ok(RangeAllocation::create(allocation, RangeAllocator::new(capacity)))
    }

    pub fn destroy(self, buffer_factory: &ManagedBufferFactory) -> Result<()> {
        buffer_factory.destroy_buffer(self.index.try_unwrap()?.allocation)?;
        buffer_factory.destroy_buffer(self.submesh.try_unwrap()?.allocation)?;
        buffer_factory.destroy_buffer(self.mesh_vertex.try_unwrap()?.allocation)?;
        buffer_factory.destroy_buffer(self.mesh_vertex_attribute.try_unwrap()?.allocation)?;
        buffer_factory.destroy_buffer(self.mesh_vertex_skin.try_unwrap()?.allocation)?;
        buffer_factory.destroy_buffer(self.mesh_bone.try_unwrap()?.allocation)?;
        buffer_factory.destroy_buffer(self.skeleton_bone.try_unwrap()?.allocation)?;
        buffer_factory.destroy_buffer(self.animation_frame.try_unwrap()?.allocation)?;

        buffer_factory.destroy_buffer(self.mesh.try_unwrap()?.allocation)?;
        buffer_factory.destroy_buffer(self.skeleton.try_unwrap()?.allocation)?;
        buffer_factory.destroy_buffer(self.animation.try_unwrap()?.allocation)?;
        buffer_factory.destroy_buffer(self.material.try_unwrap()?.allocation)?;

        Ok(())
    }
}
