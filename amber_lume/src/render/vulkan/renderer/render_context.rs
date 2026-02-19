use crate::render::vulkan::renderer::frame_context::FrameContext;
use crate::render::vulkan::renderer::render_targets::RenderTargets;
use crate::render::vulkan::swapchain::swapchain_context::SwapchainContext;
use anyhow::{Result, bail, anyhow};
use ash::{Device, Instance};
use ash::vk::{PhysicalDevice, Semaphore, SemaphoreCreateInfo};
use tracing::info;
use crate::render::vulkan::queue::queues::Queues;
use crate::render::vulkan::types::DynamicRenderingDevice;
use crate::resources::resource_factories::ResourceFactories;

pub struct RenderContext {
    current_frame: usize,
    frame_count: usize,

    frames: Vec<FrameContext>,
    present_semaphores: Vec<Semaphore>,

    pub render_targets: RenderTargets,

    pub dynamic_rendering: DynamicRenderingDevice,
}

impl RenderContext {
    pub fn create(
        instance: &Instance,
        device: &Device,
        resource_factories: &ResourceFactories,
        physical_device: PhysicalDevice,
        queues: &Queues,
        swapchain_context: &SwapchainContext,
        max_frame_count: usize,
    ) -> Result<Self> {
        let render_targets = RenderTargets::create(&instance, physical_device, &resource_factories.managed_image_factory, swapchain_context.extent)?;

        let image_count = swapchain_context.swapchain_images.len();
        let max_frame_count = max_frame_count.max(image_count);

        let frames_contexts = (0..max_frame_count)
            .map(|_| FrameContext::create(&device, &queues, &resource_factories.managed_buffer_factory))
            .collect::<Result<Vec<_>>>()?;

        let present_semaphores= Self::create_semaphores(&device, image_count)?;

        let dynamic_rendering = DynamicRenderingDevice::new(&instance, &device);

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

    fn create_semaphores(device: &Device, count: usize) -> Result<Vec<Semaphore>> {
        let semaphore_create_info = SemaphoreCreateInfo::default();

        (0..count)
            .map(|_| {
                let semaphore = unsafe { device.create_semaphore(&semaphore_create_info, None)? };

                Ok(semaphore)
            } )
            .collect::<Result<Vec<_>>>()
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

    pub fn destroy(
        self,
        device: &Device,
        resource_factories: &ResourceFactories,
    ) -> Result<()> {
        for frame in self.frames {
            frame.destroy(&device, &resource_factories.managed_buffer_factory)?;
        }
        for &present_semaphore in &self.present_semaphores {
            unsafe { device.destroy_semaphore(present_semaphore, None) }
        }

        self.render_targets.destroy(&resource_factories.managed_image_factory)?;

        info!("RenderContext destroyed");

        Ok(())
    }
}
