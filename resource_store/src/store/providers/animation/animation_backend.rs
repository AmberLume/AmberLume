use gpu_data::AnimationFrameGPU;
use gpu_data::AnimationGPU;
use anyhow::Result;
use rkyv::access;
use std::sync::Arc;
use rkyv::rancor::Error;
use tracing::info;
use resource_data::animation_data::ArchivedAnimationData;
use index_allocator::SliceIndex;
use index_allocator::ResourceLimits;
use gpu::BufferInfo;
use gpu::ResourceFactories;
use gpu::SliceBuffer;
use gpu::ResourceTransfer;
use resource_residency::ResourceBackend;
use index_allocator::ResourceId;
use index_allocator::Allocation;
use index_allocator::RangeAllocator;
use resource_reader::ResourceReader;
use crate::store::providers::animation::animation_backend_statistics::AnimationBackendStatistics;
use crate::store::providers::animation::animation_config::AnimationConfig;
use crate::store::providers::animation::buffer::animation_buffer::create_animation_buffer;
use crate::store::providers::animation::buffer::animation_frame_buffer::create_animation_frame_buffer;

pub struct AnimationBackend {
    resource_reader: Arc<dyn ResourceReader>,
    resource_transfer: Arc<ResourceTransfer>,
    resource_factories: Arc<ResourceFactories>,

    animation_frame_allocator: RangeAllocator,

    pub(crate) animation_buffer: SliceBuffer<AnimationGPU>,
    pub(crate) animation_frame_buffer: SliceBuffer<AnimationFrameGPU>,
}

impl AnimationBackend {
    pub(crate) fn new(
        limits: &ResourceLimits,
        resource_factories: Arc<ResourceFactories>,
        resource_reader: Arc<dyn ResourceReader>,
        resource_transfer: Arc<ResourceTransfer>,
    ) -> Result<Self> {
        let animation_frame_allocator = RangeAllocator::new(limits.max_animation_frames);

        let animation_buffer = create_animation_buffer(&resource_factories.buffer_factory, limits.max_animations)?;
        let animation_frame_buffer = create_animation_frame_buffer(&resource_factories.buffer_factory, limits.max_animation_frames)?;

        Ok(Self {
            resource_reader,
            resource_transfer,
            resource_factories,

            animation_frame_allocator,

            animation_buffer,
            animation_frame_buffer,
        })
    }

    fn upload_animation(&self, resource_id: ResourceId, data: AnimationGPU) -> Result<()> {
        self.resource_transfer.load_buffer_at(
            &self.animation_buffer.slice_at(SliceIndex::from(resource_id.inner)),
            &[data],
        )?;

        info!("Uploaded Animation: index: {}, data: {:?}", resource_id.inner, data);

        Ok(())
    }

    fn upload_animation_frames(&self, resource_id: ResourceId, data: &[AnimationFrameGPU]) -> Result<()> {
        self.resource_transfer.load_buffer_at(
            &self.animation_frame_buffer.slice_at(SliceIndex::from(resource_id.inner)),
            &data,
        )?;

        info!("Uploaded AnimationFrames: index: {}, count: {:?}", resource_id.inner, data.len());

        Ok(())
    }
}

pub struct AnimationHandle {
    pub name: String,

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

                let frames_allocation = self.animation_frame_allocator.allocate(frames.len() as u32).unwrap();

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

                    duration,
                    
                    frames_allocation,
                })
            }
        }
    }

    fn erase(&self, _id: &ResourceId) -> Result<()> {
        Ok(())
    }

    fn statistics(&self) -> Self::Statistics {
        Self::Statistics {
            frames: self.animation_frame_allocator.statistics(),
        }
    }

    fn destroy_resource(&self, resource: Self::Output) -> Result<()> {
        self.animation_frame_allocator.release(resource.frames_allocation);

        info!(
            "Destroyed Animation: {}, allocation [{}..+{}]",
            resource.name,
            resource.frames_allocation.offset,
            resource.frames_allocation.size,
        );

        Ok(())
    }

    fn destroy(self) -> Result<()> {
        self.resource_factories.buffer_factory.destroy_buffer(self.animation_buffer.into_managed_buffer())?;
        self.resource_factories.buffer_factory.destroy_buffer(self.animation_frame_buffer.into_managed_buffer())?;

        Ok(())
    }
}
