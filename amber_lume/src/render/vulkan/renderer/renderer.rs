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
use crate::render::vulkan::render_pass::collider::collider_render_pass::ColliderRenderPass;
use crate::render::vulkan::render_pass::collider_culling_pass::collider_culling_render_pass::ColliderCullingRenderPass;
use crate::render::vulkan::resource_context::ResourceContext;
use crate::render::vulkan::render_pass::culling_pass::culling_render_pass::CullingRenderPass;

const MAX_FRAMES_IN_FLIGHT: usize = 3;

pub struct Renderer {
    render_context: RenderContext,

    render_passes: Vec<Box<dyn RenderPass>>,
}

impl Renderer {
    pub fn create(
        vulkan_context: &VulkanContext,
        device_context: &mut DeviceContext,
        resource_context: &ResourceContext,
        swapchain_context: &SwapchainContext,
        resource_hub: Arc<ResourceHub>,
    ) -> Result<Self> {
        let render_context = RenderContext::create(
            &vulkan_context,
            device_context,
            &swapchain_context,
            MAX_FRAMES_IN_FLIGHT,
        )?;

        let culling_render_pass = CullingRenderPass::create(&resource_context, resource_hub.clone())?;
        let collider_culling_render_pass = ColliderCullingRenderPass::create(&resource_context, resource_hub.clone())?;
        let depth_render_pass = DepthRenderPass::create(&resource_context, &render_context, resource_hub.clone())?;
        let main_render_pass = MainRenderPass::create(
            &resource_context,
            &swapchain_context,
            &render_context,
            resource_hub.clone()
        )?;
        let collider_render_pass = ColliderRenderPass::create(
            &resource_context,
            &swapchain_context,
            &render_context,
            resource_hub.clone()
        )?;

        let render_passes: Vec<Box<dyn RenderPass>> = vec![
            Box::new(culling_render_pass),
            Box::new(collider_culling_render_pass),
            Box::new(depth_render_pass),
            Box::new(main_render_pass),
            Box::new(collider_render_pass),
        ];

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
        self.render_context.setup(&vulkan_context, device_context, &swapchain_context)?;

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
        let frame_context = self.render_context.get_frame(frame_index)?;

        unsafe { device.wait_for_fences(&[frame_context.fence], true, u64::MAX)? };

        let (image_index, suboptimal) = match unsafe {
            swapchain_context.loader.acquire_next_image(
                swapchain_context.handle,
                u64::MAX,
                frame_context.acquire_semaphore,
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

        let present_semaphore = self.render_context.get_present_semaphore(image_index)?;

        let render_pass_context = RenderPassContext::create(
            &device_context,
            &swapchain_context,
            &self.render_context,
            &frame_context.command_recording,
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

        render_pass_context.finalize();

        render_pass_context.end_command_recording()?;

        let wait_semaphores = [frame_context.acquire_semaphore];
        let signal_semaphores = [present_semaphore];
        let wait_stages = [PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let submit_info = SubmitInfo::default()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&wait_stages)
            .command_buffers(slice::from_ref(
                &frame_context.command_recording.command_buffer,
            ))
            .signal_semaphores(&signal_semaphores);

        unsafe { device_context.device.reset_fences(&[frame_context.fence])? };

        device_context.queues.submit_graphics(submit_info, frame_context.fence)?;

        let wait_semaphores = [present_semaphore];
        let swapchains = [swapchain_context.handle];
        let image_indices = [image_index];
        let present_info = PresentInfoKHR::default()
            .wait_semaphores(&wait_semaphores)
            .swapchains(&swapchains)
            .image_indices(&image_indices);

        let present_result = device_context.queues.present(&swapchain_context, present_info);

        if suboptimal
            || matches!(
                present_result,
                Err(vk::Result::ERROR_OUT_OF_DATE_KHR | vk::Result::ERROR_SURFACE_LOST_KHR)
            )
            || present_result == Ok(true)
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
        self.render_passes.clear();

        self.render_context.destroy(device_context)?;

        Ok(())
    }
}
