use crate::vulkan::command_recording::CommandRecording;
use crate::vulkan::instance_surface::InstanceSurface;
use crate::vulkan::logical_device::LogicalDevice;
use crate::vulkan::physical_device_choice::PhysicalDeviceChoice;
use crate::vulkan::queue_families::QueueFamilies;
use crate::vulkan::queue_set::QueueSet;
use crate::vulkan::render_targets::RenderTargets;
use crate::vulkan::swapchain::Swapchain;
use crate::vulkan::sync_primitives::SyncPrimitives;
use anyhow::Result;
use ash::{Entry, vk};
use tracing::info;
use vk::Fence;
use winit::window::Window;

pub struct VkContext {
    entry: Entry,
    instance_surface: InstanceSurface,

    physical_device_choice: PhysicalDeviceChoice,
    queue_families: QueueFamilies,

    logical_device: LogicalDevice,
    queue_set: QueueSet,

    swapchain: Swapchain,
    render_targets: RenderTargets,
    command_recording: CommandRecording,
    sync_primitives: SyncPrimitives,

    clear: [f32; 4],
}

impl VkContext {
    pub fn new(window: &Window, clear: [f32; 4]) -> Result<Self> {
        let entry = Entry::linked();
        let instance_surface = InstanceSurface::create(&entry, window)?;

        let physical_device_choice = PhysicalDeviceChoice::pick(&instance_surface)?;
        let queue_families = QueueFamilies::find(&instance_surface, physical_device_choice.device)?;

        let logical_device = LogicalDevice::create(
            &instance_surface,
            physical_device_choice.device,
            &queue_families,
        )?;
        let queue_set = QueueSet::get(&logical_device, &queue_families);

        let swapchain =
            Swapchain::create(&instance_surface, &logical_device, &queue_families, window)?;
        let render_targets = RenderTargets::create(
            &logical_device,
            swapchain.format,
            &swapchain.image_views,
            swapchain.extent,
        )?;
        let command_recording = CommandRecording::allocate_and_record(
            &logical_device,
            queue_families.graphics,
            render_targets.render_pass,
            &render_targets.framebuffers,
            swapchain.extent,
            clear,
        )?;
        let sync_primitives = SyncPrimitives::create(&logical_device)?;

        info!("VkContext is ready");

        let vk_context = Self {
            entry,
            instance_surface,
            physical_device_choice,
            queue_families,
            logical_device,
            queue_set,
            swapchain,
            render_targets,
            command_recording,
            sync_primitives,
            clear,
        };

        Ok(vk_context)
    }

    pub fn recreate_swapchain(&mut self, window: &Window) -> Result<()> {
        unsafe {
            self.logical_device.device.device_wait_idle()?;
        }
        self.render_targets.destroy(&self.logical_device);
        self.swapchain.destroy(&self.logical_device);

        self.swapchain = Swapchain::create(
            &self.instance_surface,
            &self.logical_device,
            &self.queue_families,
            window,
        )?;
        self.render_targets = RenderTargets::create(
            &self.logical_device,
            self.swapchain.format,
            &self.swapchain.image_views,
            self.swapchain.extent,
        )?;
        self.command_recording.reallocate_and_record(
            &self.logical_device,
            self.render_targets.render_pass,
            &self.render_targets.framebuffers,
            self.swapchain.extent,
        )?;
        Ok(())
    }

    pub fn draw(&mut self, _window: &Window) -> Result<()> {
        let device = &self.logical_device.device;
        unsafe {
            device.wait_for_fences(&[self.sync_primitives.in_flight], true, u64::MAX)?;
            device.reset_fences(&[self.sync_primitives.in_flight])?;
        }

        let (image_index, subopt) = unsafe {
            self.swapchain.loader.acquire_next_image(
                self.swapchain.handle,
                u64::MAX,
                self.sync_primitives.image_available,
                Fence::null(),
            )
        }?;

        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let submit = vk::SubmitInfo::default()
            .wait_semaphores(std::slice::from_ref(&self.sync_primitives.image_available))
            .wait_dst_stage_mask(&wait_stages)
            .command_buffers(std::slice::from_ref(
                &self.command_recording.buffers[image_index as usize],
            ))
            .signal_semaphores(std::slice::from_ref(&self.sync_primitives.render_finished));
        unsafe {
            device.queue_submit(
                self.queue_set.graphics,
                std::slice::from_ref(&submit),
                self.sync_primitives.in_flight,
            )?;
        }

        let swaps = [self.swapchain.handle];
        let idx = [image_index];
        let present = vk::PresentInfoKHR::default()
            .wait_semaphores(std::slice::from_ref(&self.sync_primitives.render_finished))
            .swapchains(&swaps)
            .image_indices(&idx);
        let res = unsafe {
            self.swapchain
                .loader
                .queue_present(self.queue_set.present, &present)
        };

        if matches!(res, Err(vk::Result::ERROR_OUT_OF_DATE_KHR))
            || res.as_ref() == Ok(&true)
            || subopt
        {
            // no-op, событие resize вызовет recreate в App
        }
        Ok(())
    }
}

impl Drop for VkContext {
    fn drop(&mut self) {
        unsafe {
            self.logical_device.device.device_wait_idle().ok();
        }

        self.sync_primitives.destroy(&self.logical_device);
        self.command_recording.destroy(&self.logical_device);
        self.render_targets.destroy(&self.logical_device);
        self.swapchain.destroy(&self.logical_device);
        self.logical_device.destroy();
        self.instance_surface.destroy();
    }
}
