use gpu_data::AnimationFrameGPU;
use gpu_data::AnimationGPU;
use anyhow::{Context, Result};
use dashmap::DashMap;
use std::sync::Arc;
use tracing::info;
use gpu::ResourceTransfer;
use resource_residency::ResourceBackend;
use index_allocator::ResourceId;
use crate::store::animation::backend::animation_backend_statistics::AnimationBackendStatistics;
use crate::store::animation::backend::managed_animation::ManagedAnimation;
use crate::store::animation::backend::animation_config::AnimationConfig;
use gpu::RangeAllocation;
use gpu::SingleAllocation;

pub struct AnimationBackend {
    resource_transfer: Arc<ResourceTransfer>,

    animation: Arc<SingleAllocation<AnimationGPU>>,
    animation_frame: Arc<RangeAllocation<AnimationFrameGPU>>,

    animations: DashMap<ResourceId, ManagedAnimation>,
}

impl AnimationBackend {
    pub(crate) fn new(
        resource_transfer: Arc<ResourceTransfer>,
        animation: Arc<SingleAllocation<AnimationGPU>>,
        animation_frame: Arc<RangeAllocation<AnimationFrameGPU>>,
    ) -> Self {
        Self {
            resource_transfer,

            animation,
            animation_frame,

            animations: DashMap::new(),
        }
    }

    pub fn with_animation<R>(&self, id: ResourceId, action: impl FnOnce(&ManagedAnimation) -> R) -> Option<R> {
        self.animations.get(&id).map(|animation| action(animation.value()))
    }

    fn upload_animation(&self, id: ResourceId, data: AnimationGPU) -> Result<()> {
        self.resource_transfer.load_buffer_at(
            self.animation.at(id.inner),
            &[data],
        )
    }
}

impl ResourceBackend for AnimationBackend {
    type Config = AnimationConfig;
    type Output = ();
    type Statistics = AnimationBackendStatistics;

    fn reserve(&self, id: &ResourceId) -> Result<()> {
        self.erase(id)
    }

    fn create(
        &self,
        id: &ResourceId,
        config: Self::Config,
    ) -> Result<Self::Output> {
        let AnimationConfig {
            name,
            skeleton,

            duration,
            fps,
            bone_count,
            frame_count,

            frames,
        } = config;

        let frames_allocation = self.animation_frame.allocator.allocate(frames.len() as u32)
            .with_context(|| format!("Failed to allocate {} animation frames", frames.len()))?;

        let record = AnimationGPU::create(
            frames_allocation.offset,

            bone_count,
            frame_count,
            duration,
            fps,
        );

        let uploaded = self.resource_transfer
            .load_buffer_at(
                self.animation_frame.slice(frames_allocation.offset, frames_allocation.size),
                &frames,
            )
            .and_then(|_| self.upload_animation(*id, record));

        if let Err(error) = uploaded {
            self.animation_frame.allocator.release(frames_allocation);

            return Err(error);
        }

        self.animations.insert(*id, ManagedAnimation {
            name,
            skeleton,

            duration,

            frames_allocation,
        });

        info!("Uploaded animation: index: {}, data: {:?}", id.inner, record);

        Ok(())
    }

    fn erase(&self, id: &ResourceId) -> Result<()> {
        if let Some((_, animation)) = self.animations.remove(id) {
            self.animation_frame.allocator.release(animation.frames_allocation);

            info!(
                "Destroyed animation: index: {}, name: {}, allocation: [{}..+{}]",
                id.inner, animation.name, animation.frames_allocation.offset, animation.frames_allocation.size,
            );
        }

        self.upload_animation(*id, AnimationGPU::EMPTY)
    }

    fn statistics(&self) -> Self::Statistics {
        Self::Statistics {
            frames: self.animation_frame.allocator.statistics(),
        }
    }
}
