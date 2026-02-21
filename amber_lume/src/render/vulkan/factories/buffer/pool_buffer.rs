use anyhow::Result;
use ash::vk::{DeviceAddress, DeviceSize};
use crate::render::vulkan::factories::buffer::managed_buffer::ManagedBuffer;

pub struct PoolBuffer { 
    pub handle: ManagedBuffer,

    pub item_size: DeviceSize,
    pub chunk_capacity: u32,
}

impl PoolBuffer {
    pub fn handle(
        handle: ManagedBuffer,
        item_size: DeviceSize,
        chunk_capacity: u32,
    ) -> Self {
        Self {
            handle,
            
            item_size,
            chunk_capacity,
        }
    }

    pub fn replace_with<T>(&self, data: &[T]) -> Result<()> {
        self.handle.stage(0, data)
    }
    
    pub fn capacity(&self) -> usize {
        (self.handle.size / self.item_size) as usize
    }

    pub fn ptr_to(&self, index: u32) -> DeviceAddress {
        self.handle.device_address.unwrap() + self.offset_to(index)
    }

    pub fn ptr_to_chunk(&self, index: u32) -> DeviceAddress {
        self.handle.device_address.unwrap() + self.offset_to_chunk(index)
    }

    pub fn offset_to(&self, index: u32) -> DeviceAddress {
        self.item_size * index as DeviceAddress
    }

    pub fn offset_to_chunk(&self, index: u32) -> DeviceAddress {
        self.item_size * self.chunk_capacity as DeviceAddress * index as DeviceAddress
    }
}
