use crate::resources::alpaca_resource_reader::alpaca_resource_reader::AlpacaResourceReader;
use anyhow::Result;
use rkyv::access;
use std::sync::Arc;
use rkyv::rancor::Error;
use tracing::info;
use builder::data::animation_data::ArchivedAnimationData;
use crate::ids::SliceIndex;
use crate::limits::renderer_limits::RendererLimits;
use crate::render::factories::buffer::builder::buffer_info::BufferInfo;
use crate::render::factories::buffer::slice_buffer::slice_buffer::SliceBuffer;
use crate::render::factories::resource_factories::ResourceFactories;
use crate::render::resources::resource_loader::ResourceLoader;
use crate::resources::dynamic::animation::animation_backend_statistics::AnimationBackendStatistics;
use crate::resources::dynamic::animation::animation_config::AnimationConfig;
use crate::resources::dynamic::animation::buffer::animation_buffer::{create_animation_buffer, AnimationGPU};
use crate::resources::dynamic::animation::buffer::animation_frame_buffer::{create_animation_frame_buffer, AnimationFrameGPU};
use crate::resources::dynamic::resource_backend::ResourceBackend;
use crate::resources::dynamic::resource_provider::ResourceId;
use crate::resources::range_allocator::range_allocator::{Allocation, RangeAllocator};

pub struct AnimationBackend {
    resource_factories: Arc<ResourceFactories>,
    alpaca_resource_reader: Arc<AlpacaResourceReader>,
    resource_loader: Arc<ResourceLoader>,

    animation_frame_allocator: RangeAllocator,

    pub animation_buffer: SliceBuffer<AnimationGPU>,
    pub animation_frame_buffer: SliceBuffer<AnimationFrameGPU>,
}

impl AnimationBackend {
    pub fn new(
        renderer_limits: &RendererLimits,
        resource_factories: Arc<ResourceFactories>,
        alpaca_resource_reader: Arc<AlpacaResourceReader>,
        resource_loader: Arc<ResourceLoader>,
    ) -> Result<Self> {
        let animation_frame_allocator = RangeAllocator::new(
            renderer_limits.render_resource_limits.max_animation_frames,
        );

        let animation_buffer = create_animation_buffer(
            &resource_factories.buffer_factory,
            renderer_limits.render_resource_limits.max_animations,
        )?;
        let animation_frame_buffer = create_animation_frame_buffer(
            &resource_factories.buffer_factory,
            renderer_limits.render_resource_limits.max_animation_frames,
        )?;

        Ok(Self {
            resource_factories,
            alpaca_resource_reader,
            resource_loader,

            animation_frame_allocator,

            animation_buffer,
            animation_frame_buffer,
        })
    }

    fn upload_animation(&self, resource_id: ResourceId, data: AnimationGPU) -> Result<()> {
        self.resource_loader.load_buffer_at(
            &self.animation_buffer.slice_at(SliceIndex::from(resource_id)),
            &[data],
        )?;

        info!("Uploaded Animation: index: {}, data: {:?}", resource_id, data);

        Ok(())
    }

    fn upload_animation_frames(&self, resource_id: ResourceId, data: &[AnimationFrameGPU]) -> Result<()> {
        self.resource_loader.load_buffer_at(
            &self.animation_frame_buffer.slice_at(SliceIndex::from(resource_id)),
            &data,
        )?;

        info!("Uploaded AnimationFrames: index: {}, count: {:?}", resource_id, data.len());

        Ok(())
    }
}

pub struct AnimationHandle {
    pub name: String,

    pub bone_count: u32,
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
                let bytes = self.alpaca_resource_reader.get_resource(&resource_key)?;
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

                self.upload_animation_frames(frames_allocation.offset, &frames)?;

                Ok(AnimationHandle {
                    name,

                    bone_count,
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
