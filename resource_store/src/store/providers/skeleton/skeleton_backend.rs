use gpu_data::SkeletonBoneGPU;
use gpu_data::SkeletonGPU;
use gpu::ResourceTransfer;
use index_allocator::Allocation;
use resource_residency::ResourceBackend;
use index_allocator::ResourceId;
use gpu::RangeAllocation;
use gpu::SingleAllocation;
use crate::store::providers::skeleton::skeleton_backend_statistics::SkeletonBackendStatistics;
use crate::store::providers::skeleton::skeleton_config::SkeletonConfig;
use anyhow::{Context, Result};
use rkyv::access;
use rkyv::rancor::Error;
use std::sync::Arc;
use tracing::info;
use resource_data::skeleton_data::ArchivedSkeletonData;
use resource_reader::ResourceReader;

pub struct SkeletonBackend {
    resource_reader: Arc<dyn ResourceReader>,
    resource_transfer: Arc<ResourceTransfer>,

    skeleton: Arc<SingleAllocation<SkeletonGPU>>,
    skeleton_bone: Arc<RangeAllocation<SkeletonBoneGPU>>,
}

impl SkeletonBackend {
    pub(crate) fn new(
        skeleton: Arc<SingleAllocation<SkeletonGPU>>,
        skeleton_bone: Arc<RangeAllocation<SkeletonBoneGPU>>,
        resource_reader: Arc<dyn ResourceReader>,
        resource_transfer: Arc<ResourceTransfer>,
    ) -> Result<Self> {
        Ok(Self {
            resource_reader,
            resource_transfer,

            skeleton,
            skeleton_bone,
        })
    }

    fn upload_skeleton(&self, resource_id: ResourceId, data: SkeletonGPU) -> Result<()> {
        self.resource_transfer.load_buffer_at(
            self.skeleton.at(resource_id.inner),
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
            self.skeleton_bone.slice(resource_id.inner, data.len() as u32),
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
                    .map(|archived_bone| SkeletonBoneGPU::create(archived_bone.parent_index.to_native()))
                    .collect::<Vec<_>>();

                let bones_allocation = self.skeleton_bone.allocator.allocate(bones.len() as u32)
                    .with_context(|| format!("Failed to allocate {} skeleton bones", bones.len()))?;

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

    fn erase(&self, id: &ResourceId) -> Result<()> {
        self.upload_skeleton(*id, SkeletonGPU::create(0, 0))?;

        Ok(())
    }

    fn statistics(&self) -> Self::Statistics {
        Self::Statistics {
            bone: self.skeleton_bone.allocator.statistics(),
        }
    }

    fn destroy_resource(&self, resource: Self::Output) -> Result<()> {
        self.skeleton_bone.allocator.release(resource.bones_allocation);

        info!(
            "Destroyed skeleton: {}, allocation [{}..+{}]",
            resource.name, resource.bones_allocation.offset, resource.bones_allocation.size,
        );

        Ok(())
    }
}
