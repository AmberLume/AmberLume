use ash::vk::{Buffer, DeviceAddress, DeviceSize};
use crate::virtual_buffer::buffer_blueprint::BufferBlueprint;

pub enum BufferResourceEntry {
    Transient {
        label: &'static str,
        blueprint: BufferBlueprint,
        base_offset: Option<DeviceSize>,
    },
    Imported {
        buffer: Buffer,
        offset: DeviceSize,
        size: DeviceSize,
        device_address: DeviceAddress,
        mapped_ptr: *mut u8,
    },
}

impl BufferResourceEntry {
    pub fn transient(label: &'static str, blueprint: BufferBlueprint) -> Self {
        Self::Transient {
            label,
            blueprint,
            base_offset: None,
        }
    }

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
