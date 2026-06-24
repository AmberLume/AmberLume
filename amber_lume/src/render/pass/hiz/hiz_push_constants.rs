use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};
use crate::limits::MAX_HIZ_MIPS;
use crate::render::render_graph::virtual_buffer::physical_buffer::PhysicalBuffer;

#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone)]
pub struct HiZPushConstants {
    pub counter_buffer_device_address: DeviceAddress,

    pub depth_descriptor_id: u32,
    pub mip_count: u32,
    pub width: u32,
    pub height: u32,

    pub storage_ids: [u32; MAX_HIZ_MIPS],
}

impl HiZPushConstants {
    pub fn create(
        counter_buffer: PhysicalBuffer,
        depth_descriptor_id: u32,
        mip_count: u32,
        width: u32,
        height: u32,
        storage_ids: [u32; MAX_HIZ_MIPS],
    ) -> Self {
        Self {
            counter_buffer_device_address: counter_buffer.device_address,

            depth_descriptor_id,
            mip_count,
            width,
            height,

            storage_ids,
        }
    }
}
