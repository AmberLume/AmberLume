use crate::factories::buffer::buffer_range::buffer_range::BufferRange;
use std::ptr::null_mut;
use ash::vk::{Buffer, DeviceAddress, DeviceSize};
use gpu_allocator::vulkan::Allocation;

pub struct ManagedBuffer {
    pub label: &'static str,
    pub handle: Buffer,

    pub allocation: Allocation,

    pub size: DeviceSize,
    
    pub device_address: DeviceAddress,
}

impl ManagedBuffer {
    pub fn create(
        label: &'static str,
        handle: Buffer,
        allocation: Allocation,
        size: DeviceSize,
        device_address: DeviceAddress,
    ) -> Self {
        Self {
            label,
            handle,
            allocation,
            size,
            device_address,
        }
    }

    pub fn range(&self, offset: DeviceSize, size: DeviceSize) -> BufferRange {
        assert!(
            offset + size <= self.size,
            "Buffer '{}' range {}..{} exceeds size {}",
            self.label,
            offset,
            offset + size,
            self.size,
        );

        BufferRange::create(
            self.label,
            self.handle,
            offset,
            size,
            self.device_address + offset,
            self.allocation.mapped_ptr()
                .map(|ptr| unsafe { (ptr.as_ptr() as *mut u8).add(offset as usize) })
                .unwrap_or(null_mut()),
        )
    }
}
