use gpu_data::SkeletonBoneGPU;
use gpu_data::SkeletonGPU;
use gpu::ResourceTransfer;
use resource_residency::ResourceBackend;
use index_allocator::ResourceId;
use gpu::RangeAllocation;
use gpu::SingleAllocation;
use crate::store::skeleton::backend::managed_skeleton::ManagedSkeleton;
use crate::store::skeleton::backend::skeleton_backend_statistics::SkeletonBackendStatistics;
use crate::store::skeleton::backend::skeleton_config::SkeletonConfig;
use anyhow::{Context, Result};
use dashmap::DashMap;
use std::sync::Arc;
use tracing::info;

pub struct SkeletonBackend {
    resource_transfer: Arc<ResourceTransfer>,

    skeleton: Arc<SingleAllocation<SkeletonGPU>>,
    skeleton_bone: Arc<RangeAllocation<SkeletonBoneGPU>>,

    skeletons: DashMap<ResourceId, ManagedSkeleton>,
}

impl SkeletonBackend {
    pub(crate) fn new(
        resource_transfer: Arc<ResourceTransfer>,
        skeleton: Arc<SingleAllocation<SkeletonGPU>>,
        skeleton_bone: Arc<RangeAllocation<SkeletonBoneGPU>>,
    ) -> Self {
        Self {
            resource_transfer,

            skeleton,
            skeleton_bone,

            skeletons: DashMap::new(),
        }
    }

    pub fn with_skeleton<R>(&self, id: ResourceId, action: impl FnOnce(&ManagedSkeleton) -> R) -> Option<R> {
        self.skeletons.get(&id).map(|skeleton| action(skeleton.value()))
    }

    fn upload_skeleton(&self, id: ResourceId, data: SkeletonGPU) -> Result<()> {
        self.resource_transfer.load_buffer_at(
            self.skeleton.at(id.inner),
            &[data],
        )
    }
}

impl ResourceBackend for SkeletonBackend {
    type Config = SkeletonConfig;
    type Output = ();
    type Statistics = SkeletonBackendStatistics;

    fn reserve(&self, id: &ResourceId) -> Result<()> {
        self.erase(id)
    }

    fn create(&self, id: &ResourceId, config: Self::Config) -> Result<Self::Output> {
        let SkeletonConfig {
            name,

            bones,
        } = config;

        let bones_allocation = self.skeleton_bone.allocator.allocate(bones.len() as u32)
            .with_context(|| format!("Failed to allocate {} skeleton bones", bones.len()))?;

        let record = SkeletonGPU::create(bones_allocation.offset, bones_allocation.size);

        let uploaded = self.resource_transfer
            .load_buffer_at(
                self.skeleton_bone.slice(bones_allocation.offset, bones_allocation.size),
                &bones,
            )
            .and_then(|_| self.upload_skeleton(*id, record));

        if let Err(error) = uploaded {
            self.skeleton_bone.allocator.release(bones_allocation);

            return Err(error);
        }

        self.skeletons.insert(*id, ManagedSkeleton {
            name,

            bones_allocation,
        });

        info!("Uploaded skeleton: index: {}, data: {:?}", id.inner, record);

        Ok(())
    }

    fn erase(&self, id: &ResourceId) -> Result<()> {
        if let Some((_, skeleton)) = self.skeletons.remove(id) {
            self.skeleton_bone.allocator.release(skeleton.bones_allocation);

            info!(
                "Destroyed skeleton: index: {}, name: {}, allocation: [{}..+{}]",
                id.inner, skeleton.name, skeleton.bones_allocation.offset, skeleton.bones_allocation.size,
            );
        }

        self.upload_skeleton(*id, SkeletonGPU::EMPTY)
    }

    fn statistics(&self) -> Self::Statistics {
        Self::Statistics {
            bone: self.skeleton_bone.allocator.statistics(),
        }
    }
}
