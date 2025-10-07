use crate::render::vulkan::frame_sync::FrameSync;
use crate::render::vulkan::physical_device_info::PhysicalDeviceInfo;
use crate::render::vulkan::queue_families::QueueFamilies;
use crate::render::vulkan::queue_set::QueueSet;
use crate::render::vulkan::render_targets::RenderTargets;
use crate::render::vulkan::swapchain::Swapchain;
use crate::render::vulkan::vk_context::VkContext;
use crate::render::vulkan::vk_surface::VkSurface;
use anyhow::Result;
use ash::khr::swapchain;
use ash::vk::{
    DeviceCreateInfo, DeviceQueueCreateInfo, Fence, PipelineStageFlags, PresentInfoKHR, SubmitInfo,
};
use ash::{Device, vk};
use std::slice;
use std::sync::Arc;
use tracing::{info, instrument};
use winit::window::Window;

const MAX_FRAMES_IN_FLIGHT: usize = 3;

pub struct RenderContext {
    vk_context: Arc<VkContext>,
    window: Arc<Window>,

    vk_surface: VkSurface,

    physical_device_info: PhysicalDeviceInfo,
    device: Device,
    swapchain: Swapchain,

    queue_families: QueueFamilies,
    queue_set: QueueSet,
    render_targets: RenderTargets,

    frames: Vec<FrameSync>,
    current_frame: usize,

    need_recreate_swapchain: bool,
}

impl RenderContext {
    pub fn create_from(vk_context: Arc<VkContext>, window: Arc<Window>) -> Result<Self> {
        let vk_surface = VkSurface::create(vk_context.clone(), &window)?;

        let physical_device_info = vk_context
            .physical_devices
            .iter()
            .find(|physical_device| {
                physical_device
                    .is_suitable_for(&vk_context, &vk_surface)
                    .is_ok()
            })
            .unwrap()
            .clone();

        let queue_families = QueueFamilies::find(&vk_context, &vk_surface, &physical_device_info)?;

        let device = Self::create_device(&vk_context, &physical_device_info, &queue_families)?;

        let queue_set = QueueSet::get(&device, &queue_families);

        let swapchain = Swapchain::create(
            &vk_context,
            &device,
            &physical_device_info,
            &vk_surface,
            &queue_families,
            &window,
        )?;

        let render_targets = RenderTargets::create(&device, &swapchain)?;

        let mut frames = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);
        for _ in 0..MAX_FRAMES_IN_FLIGHT {
            let frame = FrameSync::create(&device, &queue_families)?;

            frames.push(frame);
        }

        info!("VkContext is ready");

        Ok(Self {
            vk_context,
            window,

            vk_surface,

            physical_device_info,
            device,
            swapchain,

            queue_families,
            queue_set,
            render_targets,

            frames,
            current_frame: 0,

            need_recreate_swapchain: false,
        })
    }

    #[instrument(level = "trace", skip_all)]
    fn create_device(
        vk_context: &VkContext,
        physical_device_info: &PhysicalDeviceInfo,
        queue_families: &QueueFamilies,
    ) -> Result<Device> {
        let unique = if queue_families.graphics == queue_families.present {
            vec![queue_families.graphics]
        } else {
            vec![queue_families.graphics, queue_families.present]
        };
        let priorities = [1.0f32];
        let device_queue_create_info: Vec<_> = unique
            .iter()
            .map(|&i| {
                DeviceQueueCreateInfo::default()
                    .queue_family_index(i)
                    .queue_priorities(&priorities)
            })
            .collect();

        let extensions = [swapchain::NAME.as_ptr()];
        info!("Created device extensions: {:?}", extensions);
        let device_create_info = DeviceCreateInfo::default()
            .queue_create_infos(&device_queue_create_info)
            .enabled_extension_names(&extensions);
        let device = unsafe {
            vk_context.instance.create_device(
                physical_device_info.handle,
                &device_create_info,
                None,
            )?
        };

        info!("Logical device created");

        Ok(device)
    }

    pub fn request_recreate_swapchain(&mut self) {
        self.need_recreate_swapchain = true;
    }

    fn wait_idle(&self) -> Result<()> {
        unsafe { self.device.device_wait_idle()? }

        Ok(())
    }

    pub fn recreate_swapchain(&mut self) -> Result<()> {
        self.wait_idle()?;

        self.render_targets.destroy(&self.device);
        self.swapchain.destroy(&self.device);

        self.swapchain.recreate(
            &self.vk_context,
            &self.device,
            &self.physical_device_info,
            &self.vk_surface,
            &self.queue_families,
            &self.window,
        )?;
        self.render_targets = RenderTargets::create(&self.device, &self.swapchain)?;

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
            self.device
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
            .reset_begin_one_time(&self.device)?;

        frame_sync.command_recording.record_pass(
            &self.device,
            self.render_targets.render_pass,
            self.render_targets.framebuffers[image_index as usize],
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
            self.device.reset_fences(&[frame_sync.fence])?;
        }
        unsafe {
            self.device.queue_submit(
                self.queue_set.graphics,
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
                .queue_present(self.queue_set.present, &present_info)
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
            self.device.device_wait_idle().ok();
        }

        for frame in &self.frames {
            frame.destroy(&self.device);
        }
        self.render_targets.destroy(&self.device);
        self.swapchain.destroy(&self.device);
    }
}
