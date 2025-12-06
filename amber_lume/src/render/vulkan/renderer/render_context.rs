use crate::render::vulkan::device_context::DeviceContext;
use crate::render::vulkan::frame_sync::FrameSync;
use crate::render::vulkan::render_targets::RenderTargets;
use crate::render::vulkan::swapchain::swapchain_context::SwapchainContext;
use crate::render::vulkan::vulkan_context::VulkanContext;
use anyhow::{Result, bail};
use ash::khr::dynamic_rendering::Device;
use tracing::info;

pub struct RenderContext {
    current_frame: usize,
    frame_count: usize,
    frames: Vec<FrameSync>,

    pub render_targets: RenderTargets,

    pub dynamic_rendering: Device,
}

impl RenderContext {
    pub fn create(
        vulkan_context: &VulkanContext,
        device_context: &DeviceContext,
        swapchain_context: &SwapchainContext,
        frame_count: usize,
    ) -> Result<Self> {
        let render_targets =
            RenderTargets::create(&vulkan_context, &device_context, &swapchain_context)?;

        let mut frames = Vec::with_capacity(frame_count);
        for _ in 0..frame_count {
            let frame = FrameSync::create(&device_context.device, &device_context.queue_families)?;

            frames.push(frame);
        }

        let dynamic_rendering = Device::new(&vulkan_context.instance, &device_context.device);

        info!("RenderContext created");

        Ok(Self {
            render_targets,

            frames,
            frame_count,
            current_frame: 0,

            dynamic_rendering,
        })
    }

    pub fn setup(
        &mut self,
        vulkan_context: &VulkanContext,
        device_context: &DeviceContext,
        swapchain_context: &SwapchainContext,
    ) -> Result<()> {
        self.render_targets =
            RenderTargets::create(&vulkan_context, &device_context, &swapchain_context)?;

        info!("RenderContext rebuilt");

        Ok(())
    }

    pub fn next_frame_index(&mut self) -> usize {
        let frame_index = self.current_frame % self.frame_count;

        self.current_frame = (self.current_frame + 1) % self.frame_count;

        frame_index
    }

    pub fn get_frame(&self, index: usize) -> Result<&FrameSync> {
        let frame = self.frames.get(index);

        if let Some(frame) = frame {
            Ok(frame)
        } else {
            bail!("Frame index out of bounds");
        }
    }

    pub fn teardown(&mut self, device_context: &DeviceContext) -> Result<()> {
        self.render_targets.destroy(&device_context)?;

        info!("RenderContext tern down");

        Ok(())
    }

    pub fn destroy(&mut self, device_context: &DeviceContext) -> Result<()> {
        for frame in &self.frames {
            frame.destroy(&device_context)?;
        }
        self.render_targets.destroy(&device_context)?;

        info!("RenderContext destroyed");

        Ok(())
    }
}
