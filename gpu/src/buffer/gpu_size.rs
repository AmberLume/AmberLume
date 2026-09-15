use ash::vk::DeviceSize;
use bytemuck::Pod;
use std::mem::size_of;

pub trait GpuSize: Pod {
    const SIZE: DeviceSize = size_of::<Self>() as DeviceSize;
}

impl<T: Pod> GpuSize for T {}
