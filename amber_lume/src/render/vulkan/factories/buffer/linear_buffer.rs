use anyhow::{Result, bail};
use ash::vk::DeviceSize;
use std::sync::atomic::{AtomicU64, Ordering};
use crate::render::vulkan::factories::buffer::managed_buffer::ManagedBuffer;

pub struct LinearBuffer {
    pub handle: ManagedBuffer,

    offset: AtomicU64,
}

impl LinearBuffer {
    pub fn handle(
        handle: ManagedBuffer,
    ) -> Self {
        Self {
            handle,

            offset: AtomicU64::new(0),
        }
    }

    pub fn allocate_space_for(
        &self,
        size: DeviceSize,
    ) -> Result<DeviceSize> {
        loop {
            let offset = self.offset.load(Ordering::Relaxed) as DeviceSize;
            let new_size = offset + size;

            if new_size > self.handle.size {
                bail!("Failed to allocate space in '{}' buffer. Buffer size is {} but tried to allocate {}.", self.handle.name, self.handle.size, new_size);
            }
            match self.offset.compare_exchange_weak(
                offset as u64,
                new_size as u64,
                Ordering::SeqCst,
                Ordering::Relaxed,
            ) {
                Ok(_) => return Ok(offset),
                Err(_) => continue,
            }
        }
    }

    pub fn stage<T>(&self, offset: DeviceSize, data: &[T]) -> Result<()> {
        self.handle.stage(offset, data)
    }

    pub fn get_offset(&self) -> DeviceSize {
        self.offset.load(Ordering::Relaxed)
    }

    pub fn reset(&self) {
        self.offset.store(0, Ordering::Relaxed);
    }
}
