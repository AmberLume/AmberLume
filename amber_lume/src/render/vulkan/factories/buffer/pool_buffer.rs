use anyhow::Result;
use ash::vk::{DeviceAddress, DeviceSize};
use crate::render::vulkan::factories::buffer::managed_buffer::ManagedBuffer;

pub struct PoolBuffer { 
    pub handle: ManagedBuffer,

    pub item_size: DeviceSize,

}

impl PoolBuffer {
    pub fn handle(
        handle: ManagedBuffer,
        item_size: DeviceSize,
    ) -> Self {
        Self {
            handle,
            
            item_size,
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

    pub fn offset_to(&self, index: u32) -> DeviceAddress {
        self.item_size * index as DeviceAddress
    }
}
