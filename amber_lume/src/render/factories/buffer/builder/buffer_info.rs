use crate::render::factories::buffer::managed_buffer::ManagedBuffer;
use ash::vk::{Buffer, DeviceSize};

pub trait BufferInfo {
    fn handle(&self) -> Buffer;
    
    fn entire_size(&self) -> DeviceSize;

    fn into_managed_buffer(self) -> ManagedBuffer;
}
