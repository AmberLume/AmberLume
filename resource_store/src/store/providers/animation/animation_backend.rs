use gpu_data::AnimationFrameGPU;
use gpu_data::AnimationGPU;
use anyhow::{Context, Result};
use rkyv::access;
use std::sync::Arc;
use rkyv::rancor::Error;
use tracing::info;
use resource_data::animation_data::ArchivedAnimationData;
use gpu::ResourceTransfer;
use resource_residency::ResourceBackend;
use resource_residency::ResRef;
use resource_residency::ResourceProvider;
use index_allocator::ResourceId;
use index_allocator::Allocation;
use resource_reader::ResourceReader;
use crate::store::providers::animation::animation_backend_statistics::AnimationBackendStatistics;
use crate::store::providers::animation::animation_config::AnimationConfig;
use gpu::RangeAllocation;
use gpu::SingleAllocation;
use crate::store::providers::skeleton::skeleton_backend::SkeletonBackend;
use crate::store::providers::skeleton::skeleton_config::SkeletonConfig;

pub struct AnimationBackend {
    resource_reader: Arc<dyn ResourceReader>,
    resource_transfer: Arc<ResourceTransfer>,
    skeleton_provider: Arc<ResourceProvider<SkeletonBackend>>,

    animation: Arc<SingleAllocation<AnimationGPU>>,
    animation_frame: Arc<RangeAllocation<AnimationFrameGPU>>,
}

impl AnimationBackend {
    pub(crate) fn new(
        animation: Arc<SingleAllocation<AnimationGPU>>,
        animation_frame: Arc<RangeAllocation<AnimationFrameGPU>>,
        resource_reader: Arc<dyn ResourceReader>,
        resource_transfer: Arc<ResourceTransfer>,
        skeleton_provider: Arc<ResourceProvider<SkeletonBackend>>,
    ) -> Result<Self> {
        Ok(Self {
            resource_reader,
            resource_transfer,
            skeleton_provider,

            animation,
            animation_frame,
        })
    }

    fn upload_animation(&self, resource_id: ResourceId, data: AnimationGPU) -> Result<()> {
        self.resource_transfer.load_buffer_at(
            self.animation.at(resource_id.inner),
            &[data],
        )?;

        info!("Uploaded Animation: index: {}, data: {:?}", resource_id.inner, data);

        Ok(())
    }

    fn upload_animation_frames(&self, resource_id: ResourceId, data: &[AnimationFrameGPU]) -> Result<()> {
        self.resource_transfer.load_buffer_at(
            self.animation_frame.slice(resource_id.inner, data.len() as u32),
            &data,
        )?;

        info!("Uploaded AnimationFrames: index: {}, count: {:?}", resource_id.inner, data.len());

        Ok(())
    }
}

pub struct AnimationHandle {
    pub name: String,
    pub skeleton: Arc<ResRef>,

    pub duration: f32,
    
    pub frames_allocation: Allocation,
}

impl ResourceBackend for AnimationBackend {
    type Config = AnimationConfig;
    type Output = AnimationHandle;
    type Statistics = AnimationBackendStatistics;

    fn create(
        &self,
        id: &ResourceId,
        config: Self::Config,
    ) -> Result<Self::Output> {
        match config {
            AnimationConfig::Alpaca { resource_key } => {
                let bytes = self.resource_reader.get_resource(&resource_key)?;
                let archived = access::<ArchivedAnimationData, Error>(&bytes)?;

                let name = archived.name.to_string();
                let skeleton = self.skeleton_provider.get_or_load(SkeletonConfig::Alpaca {
                    resource_key: archived.skeleton.value.to_string(),
                })?;
                let duration: f32 = archived.duration.into();
                let fps: f32 = archived.fps.into();
                let bone_count: u32 = archived.bone_count.into();
                let frame_count: u32 = archived.frame_count.into();

                let frames = archived.keyframes.iter().map(|keyframe| {
                    AnimationFrameGPU::create(
                        keyframe.translation.map(|v| v.into()),
                        keyframe.rotation.map(|v| v.into()),
                        keyframe.scale.map(|v| v.into())
                    )
                }).collect::<Vec<_>>();

                let frames_allocation = self.animation_frame.allocator.allocate(frames.len() as u32)
                    .with_context(|| format!("Failed to allocate {} animation frames", frames.len()))?;

                self.upload_animation(*id, AnimationGPU::create(
                    frames_allocation.offset,

                    bone_count,
                    frame_count,
                    duration,
                    fps,
                ))?;

                self.upload_animation_frames(ResourceId::from(frames_allocation.offset), &frames)?;

                Ok(AnimationHandle {
                    name,
                    skeleton,

                    duration,
                    
                    frames_allocation,
                })
            }
        }
    }

    fn erase(&self, id: &ResourceId) -> Result<()> {
        self.upload_animation(*id, AnimationGPU::create(0, 0, 0, 0.0, 0.0))?;

        Ok(())
    }

    fn statistics(&self) -> Self::Statistics {
        Self::Statistics {
            frames: self.animation_frame.allocator.statistics(),
        }
    }

    fn destroy_resource(&self, resource: Self::Output) -> Result<()> {
        self.animation_frame.allocator.release(resource.frames_allocation);

        info!(
            "Destroyed Animation: {}, allocation [{}..+{}]",
            resource.name,
            resource.frames_allocation.offset,
            resource.frames_allocation.size,
        );

        Ok(())
    }
}
