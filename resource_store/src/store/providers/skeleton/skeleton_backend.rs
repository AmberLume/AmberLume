use gpu_data::SkeletonBoneGPU;
use gpu_data::SkeletonGPU;
use index_allocator::SliceIndex;
use gpu::BufferInfo;
use gpu::SliceBuffer;
use gpu::ResourceTransfer;
use index_allocator::Allocation;
use index_allocator::RangeAllocator;
use resource_residency::ResourceBackend;
use index_allocator::ResourceId;
use crate::store::providers::skeleton::buffer::skeleton_bones_buffer::create_skeleton_bone_buffer;
use crate::store::providers::skeleton::buffer::skeleton_buffer::create_skeleton_buffer;
use crate::store::providers::skeleton::skeleton_backend_statistics::SkeletonBackendStatistics;
use crate::store::providers::skeleton::skeleton_config::SkeletonConfig;
use anyhow::Result;
use rkyv::access;
use rkyv::rancor::Error;
use std::sync::Arc;
use tracing::info;
use resource_data::skeleton_data::ArchivedSkeletonData;
use index_allocator::ResourceLimits;
use gpu::ResourceFactories;
use resource_reader::ResourceReader;

pub struct SkeletonBackend {
    resource_reader: Arc<dyn ResourceReader>,
    resource_transfer: Arc<ResourceTransfer>,
    resource_factories: Arc<ResourceFactories>,

    bone_allocator: RangeAllocator,

    pub(crate) skeletons_buffer: SliceBuffer<SkeletonGPU>,
    pub(crate) skeleton_bones_buffer: SliceBuffer<SkeletonBoneGPU>,
}

impl SkeletonBackend {
    pub(crate) fn new(
        limits: &ResourceLimits,
        resource_factories: Arc<ResourceFactories>,
        resource_reader: Arc<dyn ResourceReader>,
        resource_transfer: Arc<ResourceTransfer>,
    ) -> Result<Self> {
        let bone_allocator = RangeAllocator::new(limits.max_skeleton_bones);

        let skeletons_buffer = create_skeleton_buffer(&resource_factories.buffer_factory, limits.max_skeletons)?;
        let skeleton_bones_buffer = create_skeleton_bone_buffer(&resource_factories.buffer_factory, limits.max_skeleton_bones)?;

        Ok(Self {
            resource_reader,
            resource_transfer,
            resource_factories,

            bone_allocator,

            skeletons_buffer,
            skeleton_bones_buffer,
        })
    }

    fn upload_skeleton(&self, resource_id: ResourceId, data: SkeletonGPU) -> Result<()> {
        self.resource_transfer.load_buffer_at(
            &self
                .skeletons_buffer
                .slice_at(SliceIndex::from(resource_id.inner)),
            &[data],
        )?;

        info!("Uploaded Skeleton: index: {}, data: {:?}",resource_id.inner, data);

        Ok(())
    }

    fn upload_skeleton_bones(
        &self,
        resource_id: ResourceId,
        data: &[SkeletonBoneGPU],
    ) -> Result<()> {
        self.resource_transfer.load_buffer_at(
            &self
                .skeleton_bones_buffer
                .slice_at(SliceIndex::from(resource_id.inner)),
            &data,
        )?;

        info!("Uploaded SkeletonBones: index: {}, count: {:?}",resource_id.inner,data.len());

        Ok(())
    }
}

pub struct ManagedSkeleton {
    pub name: String,

    pub bones_allocation: Allocation,
}

impl ResourceBackend for SkeletonBackend {
    type Config = SkeletonConfig;
    type Output = ManagedSkeleton;
    type Statistics = SkeletonBackendStatistics;

    fn create(&self, id: &ResourceId, config: Self::Config) -> Result<Self::Output> {
        match config {
            SkeletonConfig::Alpaca { resource_key } => {
                let skeleton_bytes = self.resource_reader.get_resource(&resource_key)?;
                let archived_skeleton_data =
                    access::<ArchivedSkeletonData, Error>(&skeleton_bytes)?;

                let name = archived_skeleton_data.name.to_string();

                let bones = archived_skeleton_data
                    .bones
                    .iter()
                    .map(|archived_bone| {
                        SkeletonBoneGPU::create(
                            archived_bone.parent_index.to_native(),
                            archived_bone
                                .inverse_bind_matrix
                                .map(|s| s.map(|v| v.into())),
                        )
                    })
                    .collect::<Vec<_>>();

                let bones_allocation = self.bone_allocator.allocate(bones.len() as u32).unwrap();

                self.upload_skeleton_bones(ResourceId::from(bones_allocation.offset), &bones)?;

                self.upload_skeleton(
                    *id,
                    SkeletonGPU::create(bones_allocation.offset, bones_allocation.size),
                )?;

                Ok(ManagedSkeleton {
                    name,

                    bones_allocation,
                })
            }
            SkeletonConfig::InBuilt { name, bones } => {
                let bones_allocation = self.bone_allocator.allocate(bones.len() as u32).unwrap();

                self.upload_skeleton_bones(ResourceId::from(bones_allocation.offset), &bones)?;

                self.upload_skeleton(
                    *id,
                    SkeletonGPU::create(bones_allocation.offset, bones_allocation.size),
                )?;

                Ok(ManagedSkeleton {
                    name,

                    bones_allocation,
                })
            }
        }
    }

    fn erase(&self, _id: &ResourceId) -> Result<()> {
        // self.upload_skeleton(*id, self.default_skeleton.)?;

        Ok(())
    }

    fn statistics(&self) -> Self::Statistics {
        Self::Statistics {
            bone: self.bone_allocator.statistics(),
        }
    }

    fn destroy_resource(&self, resource: Self::Output) -> Result<()> {
        self.bone_allocator.release(resource.bones_allocation);

        info!(
            "Destroyed skeleton: {}, allocation [{}..+{}]",
            resource.name, resource.bones_allocation.offset, resource.bones_allocation.size,
        );

        Ok(())
    }

    fn destroy(self) -> Result<()> {
        self.resource_factories.buffer_factory.destroy_buffer(self.skeleton_bones_buffer.into_managed_buffer())?;
        self.resource_factories.buffer_factory.destroy_buffer(self.skeletons_buffer.into_managed_buffer())?;

        Ok(())
    }
}
