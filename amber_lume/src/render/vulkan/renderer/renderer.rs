use crate::render::vulkan::buffer::buffer_manager::BufferManager;
use crate::render::vulkan::device_context::DeviceContext;
use crate::render::vulkan::render_pass::depth::depth_render_pass::DepthRenderPass;
use crate::render::vulkan::render_pass::main::main_render_pass::MainRenderPass;
use crate::render::vulkan::render_pass::render_pass::RenderPass;
use crate::render::vulkan::render_pass::render_pass_context::RenderPassContext;
use crate::render::vulkan::renderer::render_context::RenderContext;
use crate::render::vulkan::swapchain::swapchain_context::SwapchainContext;
use crate::render::vulkan::vulkan_context::VulkanContext;
use crate::resources::resource_hub::ResourceHub;
use crate::snapshot_handler::world_snapshot::WorldSnapshot;
use anyhow::Result;
use ash::vk;
use ash::vk::{Fence, PipelineStageFlags, PresentInfoKHR, SubmitInfo};
use std::slice;
use std::sync::Arc;
use tracing::info;

const MAX_FRAMES_IN_FLIGHT: usize = 3;

pub struct Renderer {
    render_context: RenderContext,

    render_passes: Vec<Box<dyn RenderPass>>,
}

impl Renderer {
    pub fn create(
        vulkan_context: &VulkanContext,
        device_context: &mut DeviceContext,
        swapchain_context: &SwapchainContext,
        resource_hub: Arc<ResourceHub>,
        buffer_manager: &BufferManager,
    ) -> Result<Self> {
        let render_context = RenderContext::create(
            &vulkan_context,
            device_context,
            &swapchain_context,
            MAX_FRAMES_IN_FLIGHT,
        )?;

        let depth_render_pass =
            DepthRenderPass::create(&render_context, resource_hub.clone(), &buffer_manager)?;
        let main_render_pass = MainRenderPass::create(
            &swapchain_context,
            &render_context,
            resource_hub.clone(),
            &buffer_manager,
        )?;

        let render_passes: Vec<Box<dyn RenderPass>> =
            vec![Box::new(depth_render_pass), Box::new(main_render_pass)];

        Ok(Self {
            render_context,

            render_passes,
        })
    }

    pub fn teardown(&mut self, device_context: &mut DeviceContext) -> Result<()> {
        self.render_context.teardown(device_context)?;

        Ok(())
    }

    pub fn setup(
        &mut self,
        vulkan_context: &VulkanContext,
        device_context: &mut DeviceContext,
        swapchain_context: &SwapchainContext,
    ) -> Result<()> {
        self.render_context
            .setup(&vulkan_context, device_context, &swapchain_context)?;

        info!("Renderer rebuilt");

        Ok(())
    }

    pub fn render_frame(
        &mut self,
        device_context: &DeviceContext,
        swapchain_context: &SwapchainContext,
        world_snapshot: Arc<WorldSnapshot>,
    ) -> Result<()> {
        let device = &device_context.device;

        let frame_index = self.render_context.next_frame_index();
        let frame_sync = self.render_context.get_frame(frame_index)?;

        unsafe { device.wait_for_fences(&[frame_sync.fence], true, u64::MAX)? };

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

                swapchain_context.set_is_out_of_date(true);

                return Ok(());
            }
            Err(e) => return Err(e.into()),
        };

        let render_pass_context = RenderPassContext::create(
            &device_context,
            &swapchain_context,
            &self.render_context,
            &frame_sync.command_recording,
            image_index as usize,
            world_snapshot.clone(),
        )?;

        render_pass_context.begin_command_recording()?;

        for render_pass in &self.render_passes {
            let is_enabled = render_pass.is_enabled();

            if is_enabled {
                render_pass.begin_record_commands(&render_pass_context)?;
                render_pass.record_commands(&render_pass_context)?;
                render_pass.end_record_commands(&render_pass_context)?;
            }
        }

        render_pass_context.end_command_recording()?;

        let wait_stages = [PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let submit_info = SubmitInfo::default()
            .wait_semaphores(slice::from_ref(&frame_sync.image_available))
            .wait_dst_stage_mask(&wait_stages)
            .command_buffers(slice::from_ref(
                &frame_sync.command_recording.command_buffer,
            ))
            .signal_semaphores(slice::from_ref(&frame_sync.render_finished));

        unsafe { device_context.device.reset_fences(&[frame_sync.fence])? };
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

            swapchain_context.set_is_out_of_date(true);
        }

        Ok(())
    }

    pub fn destroy(&mut self, device_context: &mut DeviceContext) -> Result<()> {
        for render_pass in &self.render_passes {
            render_pass.destroy()?;
        }

        self.render_context.destroy(device_context)?;

        Ok(())
    }
}
