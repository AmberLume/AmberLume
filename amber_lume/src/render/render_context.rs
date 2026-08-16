use crate::limits::RenderLimits;
use crate::render::frame::frame_resources::FrameResources;
use crate::render::pass::depth::depth_format::find_depth_format;
use anyhow::{bail, Result};
use ash::vk::{Format, PhysicalDevice};
use ash::{Device, Instance};
use gpu::Queues;
use index_allocator::FrameIndex;
use tracing::info;

pub struct RenderContext {
    current_frame: u32,
    frame_count: u32,

    frames: Vec<FrameResources>,

    pub depth_format: Format,
}

impl RenderContext {
    pub fn create(
        instance: &Instance,
        device: &Device,
        limits: &RenderLimits,
        physical_device: PhysicalDevice,
        queues: &Queues,
    ) -> Result<Self> {
        let frames_contexts = (0..limits.frames_in_flight)
            .map(|_| FrameResources::create(&device, &queues))
            .collect::<Result<Vec<_>>>()?;

        info!("RenderContext created");

        Ok(Self {
            current_frame: 0,
            frame_count: limits.frames_in_flight,

            frames: frames_contexts,

            depth_format: find_depth_format(&instance, physical_device)?,
        })
    }

    pub fn next_frame_index(&mut self) -> FrameIndex {
        let frame_index = self.current_frame % self.frame_count;

        self.current_frame = (self.current_frame + 1) % self.frame_count;

        FrameIndex { value: frame_index }
    }

    pub fn get_frame(&self, index: FrameIndex) -> Result<&FrameResources> {
        let frame = self.frames.get(index.value as usize);

        if let Some(frame) = frame {
            Ok(frame)
        } else {
            bail!("Frame index out of bounds");
        }
    }

    pub fn destroy(self, device: &Device) -> Result<()> {
        for frame in self.frames {
            frame.destroy(&device)?;
        }

        info!("RenderContext destroyed");

        Ok(())
    }
}
