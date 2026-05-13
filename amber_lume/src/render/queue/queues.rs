use std::sync::{Arc};
use ash::Device;
use crate::render::queue::queue_families::{QueueFamilies, QueueFamily};
use ash::vk::{Fence, Queue, SharingMode, SubmitInfo};
use crate::render::utils::debug_utils::DebugUtils;
use anyhow::Result;
use parking_lot::Mutex;

#[derive(Clone)]
pub struct QueueInfo {
    pub queue: Arc<Mutex<Queue>>,
    pub family: u32,
}

pub struct Queues {
    device: Device,

    graphics: QueueInfo,
    transfer: QueueInfo,

    present: Mutex<Option<QueueInfo>>,
}

impl Queues {
    pub fn new(device: &Device, debug_utils: &DebugUtils, families: &QueueFamilies) -> Self {
        let graphics = Self::create_single_queue(device, debug_utils, families.graphics, "graphics");

        let transfer = if let Some(transfer) = families.transfer {
            if transfer.index == families.graphics.index {
                graphics.clone()
            } else {
                Self::create_single_queue(device, debug_utils, transfer, "transfer")
            }
        } else {
            graphics.clone()
        };

        Self {
            device: device.clone(),

            graphics,
            transfer,

            present: Mutex::new(None),
        }
    }

    pub fn bind_present(&self, present: QueueInfo) {
        *self.present.lock() = Some(present);
    }

    pub fn unbind_present(&self) {
        *self.present.lock() = None;
    }

    pub fn present_queue(&self) -> QueueInfo {
        self.present.lock().as_ref()
            .expect("present queue not bound")
            .clone()
    }

    pub fn sharing_mode(&self) -> SharingMode {
        let present = self.present_queue();

        if self.graphics.family == present.family {
            SharingMode::EXCLUSIVE
        } else {
            SharingMode::CONCURRENT
        }
    }

    pub fn queue_family_indices(&self, sharing_mode: SharingMode) -> Vec<u32> {
        match sharing_mode {
            SharingMode::EXCLUSIVE => vec![self.graphics.family],
            SharingMode::CONCURRENT => vec![self.graphics.family, self.present_queue().family],
            _ => unreachable!(),
        }
    }

    pub fn create_single_queue(
        device: &Device,
        debug_utils: &DebugUtils,
        queue_family: QueueFamily,
        label: &str,
    ) -> QueueInfo {
        let queue = unsafe { device.get_device_queue(queue_family.index, 0) };

        debug_utils.label(queue, &format!(
            "{}_queue(family: {}, index: 0)",
            label, queue_family.index,
        ));

        QueueInfo {
            queue: Arc::new(Mutex::new(queue)),
            family: queue_family.index,
        }
    }

    pub fn graphics_queue_family(&self) -> u32 {
        self.graphics.family
    }

    pub fn transfer_queue_family(&self) -> u32 {
        self.transfer.family
    }

    pub fn submit_graphics(&self, submit_info: SubmitInfo, completion_fence: Fence) -> Result<()> {
        let queue_guard = self.graphics.queue.lock();

        unsafe { self.device.queue_submit(*queue_guard, &[submit_info], completion_fence)? };

        Ok(())
    }

    pub fn submit_transfer(&self, submit_infos: &[SubmitInfo], completion_fence: Fence) -> Result<()> {
        let queue_guard = self.transfer.queue.lock();

        unsafe { self.device.queue_submit(*queue_guard, submit_infos, completion_fence)? }

        unsafe { self.device.wait_for_fences(&[completion_fence], true, u64::MAX)? };

        Ok(())
    }

    pub fn present_wait_idle(&self) -> Result<()> {
        self.wait_queue_idle(&self.graphics)?;

        let present = self.present.lock().clone();

        if let Some(present) = present {
            if present.family != self.graphics.family {
                self.wait_queue_idle(&present)?;
            }
        }

        Ok(())
    }

    pub fn all_wait_idle(&self) -> Result<()> {
        self.wait_queue_idle(&self.graphics)?;

        let present = self.present.lock().clone();

        if let Some(present) = present {
            if present.family != self.graphics.family {
                self.wait_queue_idle(&present)?;
            }
        }

        if self.transfer.family != self.graphics.family {
            self.wait_queue_idle(&self.transfer)?;
        }

        Ok(())
    }

    fn wait_queue_idle(&self, queue_info: &QueueInfo) -> Result<()> {
        let queue_guard = queue_info.queue.lock();

        unsafe { self.device.queue_wait_idle(*queue_guard)? };

        Ok(())
    }
}
