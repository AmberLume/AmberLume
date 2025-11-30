use crate::render::vulkan::device_context::DeviceContext;
use crate::render::vulkan::frame_sync::FrameSync;
use crate::render::vulkan::render_targets::RenderTargets;
use crate::render::vulkan::surface_provider::SurfaceProvider;
use crate::render::vulkan::swapchain::Swapchain;
use crate::render::vulkan::vk_context::VkContext;
use crate::render::vulkan::vk_surface::VkSurface;
use anyhow::Result;
use ash::khr::dynamic_rendering::Device;
use ash::vk;
use ash::vk::{Fence, PipelineStageFlags, PresentInfoKHR, SubmitInfo};
use std::slice;
use std::sync::Arc;
use tracing::info;

const MAX_FRAMES_IN_FLIGHT: usize = 3;

pub struct RenderContext {
    vk_context: Arc<VkContext>,
    vk_surface: Arc<VkSurface>,
    device_context: Arc<DeviceContext>,

    surface_provider: Arc<dyn SurfaceProvider>,

    swapchain: Swapchain,
    render_targets: RenderTargets,

    frames: Vec<FrameSync>,
    current_frame: usize,

    need_recreate_swapchain: bool,

    dynamic_rendering: Device,
}

impl RenderContext {
    pub fn create(
        vk_context: Arc<VkContext>,
        vk_surface: Arc<VkSurface>,
        device_context: Arc<DeviceContext>,
        surface_provider: Arc<dyn SurfaceProvider>,
    ) -> Result<Self> {
        let swapchain = Swapchain::create(
            &vk_context,
            &vk_surface,
            &device_context,
            &surface_provider.size(),
        )?;

        let render_targets = RenderTargets::create(&vk_context, &device_context, &swapchain)?;

        let mut frames = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);
        for _ in 0..MAX_FRAMES_IN_FLIGHT {
            let frame = FrameSync::create(&device_context.device, &device_context.queue_families)?;

            frames.push(frame);
        }

        let dynamic_rendering = Device::new(&vk_context.instance, &device_context.device);

        info!("RenderContext is ready");

        Ok(Self {
            vk_context,
            vk_surface,
            device_context,

            surface_provider,

            swapchain,
            render_targets,

            frames,
            current_frame: 0,

            need_recreate_swapchain: false,

            dynamic_rendering,
        })
    }

    pub fn request_recreate_swapchain(&mut self) {
        self.need_recreate_swapchain = true;
    }

    fn wait_idle(&self) -> Result<()> {
        unsafe { self.device_context.device.device_wait_idle()? }

        Ok(())
    }

    pub fn recreate_swapchain(&mut self) -> Result<()> {
        self.wait_idle()?;

        self.render_targets.destroy(&self.device_context.device);

        self.swapchain.recreate(
            &self.vk_context,
            &self.vk_surface,
            &self.device_context,
            &self.surface_provider.size(),
        )?;
        self.render_targets =
            RenderTargets::create(&self.vk_context, &self.device_context, &self.swapchain)?;

        self.need_recreate_swapchain = false;

        Ok(())
    }

    pub fn begin_frame(&mut self) -> Result<()> {
        if self.need_recreate_swapchain {
            return self.recreate_swapchain();
        }

        if self.swapchain.extent.width == 0 || self.swapchain.extent.height == 0 {
            return Ok(());
        }

        let frame_index = self.current_frame % MAX_FRAMES_IN_FLIGHT;
        let frame_sync = &self.frames[frame_index];

        unsafe {
            self.device_context
                .device
                .wait_for_fences(&[frame_sync.fence], true, u64::MAX)?;
        }

        let (image_index, suboptimal) = match unsafe {
            self.swapchain.loader.acquire_next_image(
                self.swapchain.handle,
                u64::MAX,
                frame_sync.image_available,
                Fence::null(),
            )
        } {
            Ok(result) => result,
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                self.request_recreate_swapchain();

                return Ok(());
            }
            Err(e) => return Err(e.into()),
        };

        frame_sync
            .command_recording
            .reset_begin_one_time(&self.device_context.device)?;

        frame_sync.command_recording.record_pass(
            &self.device_context,
            &self.dynamic_rendering,
            self.swapchain.images[image_index as usize],
            self.swapchain.image_views[image_index as usize],
            self.render_targets.depth_gpu_images[image_index as usize].image_view,
            self.swapchain.extent,
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
            self.device_context
                .device
                .reset_fences(&[frame_sync.fence])?;
        }
        let graphics_queue = self.device_context.queues.graphics();
        unsafe {
            self.device_context.device.queue_submit(
                graphics_queue.queue,
                slice::from_ref(&submit_info),
                frame_sync.fence,
            )?;
        }

        let swapchains = [self.swapchain.handle];
        let image_indices = [image_index];
        let present_info = PresentInfoKHR::default()
            .wait_semaphores(slice::from_ref(&frame_sync.render_finished))
            .swapchains(&swapchains)
            .image_indices(&image_indices);
        let present_res = unsafe {
            self.swapchain
                .loader
                .queue_present(self.device_context.queues.present().queue, &present_info)
        };

        if suboptimal
            || matches!(
                present_res,
                Err(vk::Result::ERROR_OUT_OF_DATE_KHR | vk::Result::ERROR_SURFACE_LOST_KHR)
            )
            || present_res.as_ref() == Ok(&true)
        {
            self.request_recreate_swapchain();
        }

        self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
        Ok(())
    }
}

impl Drop for RenderContext {
    fn drop(&mut self) {
        unsafe {
            self.device_context.device.device_wait_idle().ok();
        }

        for frame in &self.frames {
            frame.destroy(&self.device_context.device);
        }
        self.render_targets.destroy(&self.device_context.device);
        self.swapchain.destroy(&self.device_context.device);
        unsafe {
            self.device_context.device.destroy_device(None);
        }
    }
}
