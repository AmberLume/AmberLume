use ash::vk::{Buffer, DeviceAddress, DeviceSize};

pub enum BufferResourceEntry {
    Imported {
        buffer: Buffer,
        offset: DeviceSize,
        size: DeviceSize,
        device_address: DeviceAddress,
        mapped_ptr: *mut u8,
    },
}

impl BufferResourceEntry {
    pub fn imported(
        buffer: Buffer,
        offset: DeviceSize,
        size: DeviceSize,
        device_address: DeviceAddress,
        mapped_ptr: *mut u8,
    ) -> Self {
        Self::Imported {
            buffer,
            offset,
            size,
            device_address,
            mapped_ptr,
        }
    }
}
