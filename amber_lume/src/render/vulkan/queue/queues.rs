use std::sync::{Arc, Mutex};
use ash::Device;
use crate::render::vulkan::queue::queue_families::{QueueFamilies, QueueFamily};
use ash::vk::{Fence, PresentInfoKHR, Queue, SharingMode, SubmitInfo};
use crate::render::vulkan::debug_utils::DebugUtils;
use anyhow::Result;
use ash::prelude::VkResult;
use crate::render::vulkan::swapchain::swapchain_context::SwapchainContext;

#[derive(Clone)]
pub struct QueueInfo {
    pub queue: Arc<Mutex<Queue>>,
    pub family: u32,
}

pub struct Queues {
    device: Device,

    graphics: QueueInfo,
    present: QueueInfo,
    transfer: QueueInfo,
}

impl Queues {
    pub fn new(device: &Device, debug_utils: &DebugUtils, families: &QueueFamilies) -> Self {
        let graphics = Self::create_single_queue(device, debug_utils, families.graphics, "graphics");

        let present = if families.present.index == families.graphics.index {
            graphics.clone()
        } else {
            Self::create_single_queue(device, debug_utils, families.present, "present")
        };

        let transfer = if let Some(transfer) = families.transfer {
            if transfer.index == families.graphics.index {
                graphics.clone()
            } else if transfer.index == families.present.index {
                present.clone()
            } else {
                Self::create_single_queue(device, debug_utils, transfer, "transfer")
            }
        } else {
            graphics.clone()
        };

        Self {
            device: device.clone(),

            graphics,
            present,
            transfer,
        }
    }

    pub fn sharing_mode(&self) -> SharingMode {
        if self.graphics.family == self.present.family {
            SharingMode::EXCLUSIVE
        } else {
            SharingMode::CONCURRENT
        }
    }

    pub fn queue_family_indices(
        &self,
        sharing_mode: SharingMode,
    ) -> Vec<u32> {
        match sharing_mode {
            SharingMode::EXCLUSIVE => vec![self.graphics.family],
            SharingMode::CONCURRENT => vec![self.graphics.family, self.present.family],
            _ => unreachable!(),
        }
    }

    fn create_single_queue(
        device: &Device,
        debug_utils: &DebugUtils,
        queue_family: QueueFamily,
        label: &str,
    ) -> QueueInfo {
        Self::create_queue(device, debug_utils, queue_family.index, 0, &label)
    }

    fn create_queue(
        device: &Device,
        debug_utils: &DebugUtils,
        family_index: u32,
        queue_index: u32,
        label: &str,
    ) -> QueueInfo {
        let queue = unsafe { device.get_device_queue(family_index, queue_index) };

        debug_utils.label(queue, &format!("{}_queue(family: {}, index: {})", &label, family_index, queue_index));

        QueueInfo {
            queue: Arc::new(Mutex::new(queue)),
            family: family_index,
        }
    }

    pub fn graphics_queue_family(&self) -> u32 {
        self.graphics.family
    }

    pub fn present_queue_family(&self) -> u32 {
        self.present.family
    }

    pub fn transfer_queue_family(&self) -> u32 {
        self.transfer.family
    }

    pub fn submit_graphics(&self, submit_info: SubmitInfo, completion_fence: Fence) -> Result<()> {
        let queue_guard = self.graphics.queue.lock().unwrap();

        unsafe { self.device.queue_submit(*queue_guard, &[submit_info], completion_fence)? };

        Ok(())
    }

    pub fn present(&self, swapchain_context: &SwapchainContext, present_info: PresentInfoKHR) -> VkResult<bool> {
        let queue_guard = self.present.queue.lock().unwrap();

        unsafe { swapchain_context.loader.queue_present(*queue_guard, &present_info) }
    }

    pub fn submit_transfer(&self, submit_infos: &[SubmitInfo], completion_fence: Fence) -> Result<()> {
        let queue_guard = self.transfer.queue.lock().unwrap();

        unsafe { self.device.queue_submit(*queue_guard, submit_infos, completion_fence)? }

        unsafe { self.device.wait_for_fences(&[completion_fence], true, u64::MAX)? };

        Ok(())
    }

    pub fn present_wait_idle(&self) -> Result<()> {
        self.wait_queue_idle(&self.graphics)?;
        self.wait_queue_idle(&self.present)?;

        Ok(())
    }

    pub fn transfer_wait_idle(&self) -> Result<()> {
        self.wait_queue_idle(&self.transfer)
    }

    pub fn all_wait_idle(&self) -> Result<()> {
        self.wait_queue_idle(&self.graphics)?;
        self.wait_queue_idle(&self.present)?;
        self.wait_queue_idle(&self.transfer)?;

        Ok(())
    }

    fn wait_queue_idle(&self, queue_info: &QueueInfo) -> Result<()> {
        let queue_guard = queue_info.queue.lock().unwrap();

        unsafe { self.device.queue_wait_idle(*queue_guard)? };

        Ok(())
    }
}
