use crate::render::vulkan::device_context::DeviceContext;
use crate::render::vulkan::renderer::frame_context::FrameContext;
use crate::render::vulkan::renderer::render_targets::RenderTargets;
use crate::render::vulkan::swapchain::swapchain_context::SwapchainContext;
use crate::render::vulkan::vulkan_context::VulkanContext;
use anyhow::{Result, bail, anyhow};
use ash::khr::dynamic_rendering::Device;
use ash::vk::{Semaphore, SemaphoreCreateInfo};
use tracing::info;

pub struct RenderContext {
    current_frame: usize,
    frame_count: usize,

    frames: Vec<FrameContext>,
    present_semaphores: Vec<Semaphore>,

    pub render_targets: RenderTargets,

    pub dynamic_rendering: Device,
}

impl RenderContext {
    pub fn create(
        vulkan_context: &VulkanContext,
        device_context: &mut DeviceContext,
        swapchain_context: &SwapchainContext,
        max_frame_count: usize,
    ) -> Result<Self> {
        let render_targets = RenderTargets::create(&vulkan_context, device_context, &swapchain_context)?;

        let image_count = swapchain_context.vulkan_images.len();
        let max_frame_count = max_frame_count.max(image_count);

        let frames_contexts = (0..max_frame_count)
            .map(|_| FrameContext::create(&device_context))
            .collect::<Result<Vec<_>>>()?;

        let present_semaphores= Self::create_semaphores(&device_context, image_count)?;

        let dynamic_rendering = Device::new(&vulkan_context.instance, &device_context.device);

        info!("RenderContext created");

        Ok(Self {
            current_frame: 0,
            frame_count: max_frame_count,

            frames: frames_contexts,
            present_semaphores,

            render_targets,

            dynamic_rendering,
        })
    }

    fn create_semaphores(device_context: &DeviceContext, count: usize) -> Result<Vec<Semaphore>> {
        let device = &device_context.device;

        let semaphore_create_info = SemaphoreCreateInfo::default();

        (0..count)
            .map(|_| {
                let semaphore = unsafe { device.create_semaphore(&semaphore_create_info, None)? };

                Ok(semaphore)
            } )
            .collect::<Result<Vec<_>>>()
    }

    pub fn setup(
        &mut self,
        vulkan_context: &VulkanContext,
        device_context: &mut DeviceContext,
        swapchain_context: &SwapchainContext,
    ) -> Result<()> {
        self.current_frame = 0;

        self.render_targets = RenderTargets::create(&vulkan_context, device_context, &swapchain_context)?;

        info!("RenderContext rebuilt");

        Ok(())
    }

    pub fn next_frame_index(&mut self) -> usize {
        let frame_index = self.current_frame % self.frame_count;

        self.current_frame = (self.current_frame + 1) % self.frame_count;

        frame_index
    }

    pub fn get_frame(&self, index: usize) -> Result<&FrameContext> {
        let frame = self.frames.get(index);

        if let Some(frame) = frame {
            Ok(frame)
        } else {
            bail!("Frame index out of bounds");
        }
    }

    pub fn get_present_semaphore(&self, image_index: u32) -> Result<Semaphore> {
        self.present_semaphores
            .get(image_index as usize)
            .cloned()
            .ok_or_else(|| anyhow!("Present semaphore index out of bounds"))
    }

    pub fn teardown(&mut self, device_context: &mut DeviceContext) -> Result<()> {
        self.render_targets.destroy(device_context)?;

        info!("RenderContext tern down");

        Ok(())
    }

    pub fn destroy(&mut self, device_context: &mut DeviceContext) -> Result<()> {
        for frame in &self.frames {
            frame.destroy(&device_context)?;
        }
        for &present_semaphore in &self.present_semaphores {
            unsafe { device_context.device.destroy_semaphore(present_semaphore, None) }
        }

        self.render_targets.destroy(device_context)?;

        info!("RenderContext destroyed");

        Ok(())
    }
}
