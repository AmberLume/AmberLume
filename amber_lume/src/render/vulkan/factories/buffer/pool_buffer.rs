use anyhow::{Result, bail};
use ash::vk::DeviceSize;
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::render::vulkan::factories::buffer::managed_buffer::ManagedBuffer;

pub struct PoolBuffer { 
    pub handle: ManagedBuffer,

    pub item_size: DeviceSize,
    
    offset: AtomicUsize,
}

impl PoolBuffer {
    pub fn handle(
        handle: ManagedBuffer,
        item_size: DeviceSize,
    ) -> Self {
        Self {
            handle,
            
            item_size,
            
            offset: AtomicUsize::new(0),
        }
    }

    pub fn allocate_space_for(
        &self,
        count: usize,
    ) -> Result<usize> {
        loop {
            let offset = self.offset.load(Ordering::Relaxed);
            let new_size = offset + count;

            if new_size as DeviceSize * self.item_size > self.handle.size {
                bail!("Failed to allocate space in '{}' buffer. Buffer size is {} but tried to allocate {}.", self.handle.name, self.handle.size, new_size);
            }

            match self.offset.compare_exchange_weak(
                offset,
                new_size,
                Ordering::SeqCst,
                Ordering::Relaxed,
            ) {
                Ok(_) => return Ok(offset),
                Err(_) => continue,
            }
        }
    }

    pub fn stage<T: Copy>(&self, offset: usize, data: &[T]) -> Result<()> {
        println!("Stage offset for '{}': {}", self.handle.name, offset);
        self.handle.stage(offset as DeviceSize * self.item_size, data)
    }
    
    pub fn capacity(&self) -> usize {
        (self.handle.size / self.item_size) as usize
    }
}
