use crate::render::vulkan::device_context::DeviceContext;
use crate::render::vulkan::renderer::render_context::RenderContext;
use crate::render::vulkan::swapchain::swapchain_context::SwapchainContext;
use crate::render::vulkan::vulkan_context::VulkanContext;
use anyhow::Result;
use ash::vk;
use ash::vk::{Fence, PipelineStageFlags, PresentInfoKHR, SubmitInfo};
use std::slice;
use tracing::info;

const MAX_FRAMES_IN_FLIGHT: usize = 3;

pub struct Renderer {
    render_context: RenderContext,
}

impl Renderer {
    pub fn create(
        vulkan_context: &VulkanContext,
        device_context: &DeviceContext,
        swapchain_context: &SwapchainContext,
    ) -> Result<Self> {
        let render_context = RenderContext::create(
            &vulkan_context,
            &device_context,
            &swapchain_context,
            MAX_FRAMES_IN_FLIGHT,
        )?;

        Ok(Self { render_context })
    }

    pub fn teardown(&mut self, device_context: &DeviceContext) -> Result<()> {
        self.render_context.teardown(&device_context)?;

        Ok(())
    }

    pub fn setup(
        &mut self,
        vulkan_context: &VulkanContext,
        device_context: &DeviceContext,
        swapchain_context: &SwapchainContext,
    ) -> Result<()> {
        self.render_context
            .setup(&vulkan_context, &device_context, &swapchain_context)?;

        info!("Renderer rebuilt");

        Ok(())
    }

    pub fn render_frame(
        &mut self,
        device_context: &DeviceContext,
        swapchain_context: &SwapchainContext,
    ) -> Result<()> {
        let frame_index = self.render_context.next_frame_index();
        let frame_sync = self.render_context.get_frame(frame_index)?;

        unsafe {
            device_context
                .device
                .wait_for_fences(&[frame_sync.fence], true, u64::MAX)?
        };

        let (image_index, suboptimal) = match unsafe {
            swapchain_context.loader.acquire_next_image(
                swapchain_context.handle,
                u64::MAX,
                frame_sync.image_available,
                Fence::null(),
            )
        } {
            Ok(result) => result,
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                info!("Swapchain swapchain image out of date");
                // self.request_recreate_swapchain();

                return Ok(());
            }
            Err(e) => return Err(e.into()),
        };

        frame_sync
            .command_recording
            .reset_begin_one_time(&device_context.device)?;

        let depth_gpu_image = &self
            .render_context
            .render_targets
            .get_depth_image(image_index as usize)?;
        frame_sync.command_recording.record_pass(
            &device_context,
            &self.render_context.dynamic_rendering,
            swapchain_context.get_image(image_index as usize)?,
            swapchain_context.get_image_view(image_index as usize)?,
            depth_gpu_image.image,
            depth_gpu_image.image_view,
            depth_gpu_image.format,
            swapchain_context.extent,
        )?;

        let wait_stages = [PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let submit_info = SubmitInfo::default()
            .wait_semaphores(slice::from_ref(&frame_sync.image_available))
            .wait_dst_stage_mask(&wait_stages)
            .command_buffers(slice::from_ref(
                &frame_sync.command_recording.command_buffer,
            ))
            .signal_semaphores(slice::from_ref(&frame_sync.render_finished));

        unsafe {
            device_context.device.reset_fences(&[frame_sync.fence])?;
        }
        let graphics_queue = device_context.queues.graphics();
        unsafe {
            device_context.device.queue_submit(
                graphics_queue.queue,
                slice::from_ref(&submit_info),
                frame_sync.fence,
            )?;
        }

        let swapchains = [swapchain_context.handle];
        let image_indices = [image_index];
        let present_info = PresentInfoKHR::default()
            .wait_semaphores(slice::from_ref(&frame_sync.render_finished))
            .swapchains(&swapchains)
            .image_indices(&image_indices);
        let present_res = unsafe {
            swapchain_context
                .loader
                .queue_present(device_context.queues.present().queue, &present_info)
        };

        if suboptimal
            || matches!(
                present_res,
                Err(vk::Result::ERROR_OUT_OF_DATE_KHR | vk::Result::ERROR_SURFACE_LOST_KHR)
            )
            || present_res.as_ref() == Ok(&true)
        {
            info!("Swapchain swapchain image out of date");
            // self.request_recreate_swapchain();
        }

        Ok(())
    }

    pub fn destroy(&mut self, device_context: &DeviceContext) -> Result<()> {
        self.render_context.destroy(&device_context)?;

        Ok(())
    }
}
