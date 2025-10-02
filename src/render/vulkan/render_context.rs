use crate::render::vulkan::command_recording::CommandRecording;
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
    command_recording: CommandRecording,
}

impl RenderContext {
    pub fn create_from(
        vk_context: Arc<VkContext>,
        window: Arc<Window>,
        clear_color: [f32; 4],
    ) -> Result<Self> {
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

        let render_targets = RenderTargets::create(
            &device,
            swapchain.format,
            &swapchain.image_views,
            swapchain.extent,
        )?;
        let command_recording = CommandRecording::allocate_and_record(
            &device,
            queue_families.graphics,
            render_targets.render_pass,
            &render_targets.framebuffers,
            swapchain.extent,
            clear_color,
        )?;

        let mut frames = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);
        for _ in 0..MAX_FRAMES_IN_FLIGHT {
            let frame = FrameSync::create(&device)?;

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
            command_recording,
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

    fn wait_idle(&self) -> Result<()> {
        unsafe { self.device.device_wait_idle()? }

        Ok(())
    }

    pub fn recreate_swapchain(&mut self) -> Result<()> {
        self.wait_idle()?;
        self.render_targets.destroy(&self.device);
        self.swapchain.destroy(&self.device);

        self.swapchain = Swapchain::create(
            &self.vk_context,
            &self.device,
            &self.physical_device_info,
            &self.vk_surface,
            &self.queue_families,
            &self.window,
        )?;
        self.render_targets = RenderTargets::create(
            &self.device,
            self.swapchain.format,
            &self.swapchain.image_views,
            self.swapchain.extent,
        )?;
        self.command_recording.reallocate_and_record(
            &self.device,
            self.render_targets.render_pass,
            &self.render_targets.framebuffers,
            self.swapchain.extent,
        )?;
        Ok(())
    }

    pub fn draw(&mut self, _window: &Window) -> Result<()> {
        let device = &self.device;
        let frame_index = self.current_frame % MAX_FRAMES_IN_FLIGHT;
        let frame_sync = &self.frames[frame_index];

        unsafe {
            device.wait_for_fences(&[frame_sync.fence], true, u64::MAX)?;
            device.reset_fences(&[frame_sync.fence])?;
        }

        let (image_index, suboptimal) = unsafe {
            self.swapchain.loader.acquire_next_image(
                self.swapchain.handle,
                u64::MAX,
                frame_sync.image_available,
                Fence::null(),
            )
        }?;

        let wait_stages = [PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let submit = SubmitInfo::default()
            .wait_semaphores(slice::from_ref(&frame_sync.image_available))
            .wait_dst_stage_mask(&wait_stages)
            .command_buffers(slice::from_ref(
                &self.command_recording.buffers[image_index as usize],
            ))
            .signal_semaphores(slice::from_ref(&frame_sync.render_finished));
        unsafe {
            device.queue_submit(
                self.queue_set.graphics,
                slice::from_ref(&submit),
                frame_sync.fence,
            )?;
        }

        let swaps = [self.swapchain.handle];
        let idx = [image_index];
        let present = PresentInfoKHR::default()
            .wait_semaphores(slice::from_ref(&frame_sync.render_finished))
            .swapchains(&swaps)
            .image_indices(&idx);
        let res = unsafe {
            self.swapchain
                .loader
                .queue_present(self.queue_set.present, &present)
        };

        if matches!(res, Err(vk::Result::ERROR_OUT_OF_DATE_KHR))
            || res.as_ref() == Ok(&true)
            || suboptimal
        {
            // no-op, событие resize вызовет recreate в App
        }

        self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
        Ok(())
    }
}
