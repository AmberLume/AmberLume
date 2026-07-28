use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};
use crate::render::render_graph::virtual_buffer::physical_buffer::PhysicalBuffer;

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct CullRequestGPU {
    pub indirect_buffer_device_address: DeviceAddress,
    pub draw_count_buffer_device_address: DeviceAddress,
    pub draw_data_buffer_device_address: DeviceAddress,

    pub accept_mask: u32,

    _pad0: u32,
}

impl CullRequestGPU {
    pub fn create(
        indirect_buffer: PhysicalBuffer,
        draw_count_buffer: PhysicalBuffer,
        draw_data_buffer: PhysicalBuffer,
        accept_mask: u32,
    ) -> Self {
        Self {
            indirect_buffer_device_address: indirect_buffer.device_address,
            draw_count_buffer_device_address: draw_count_buffer.device_address,
            draw_data_buffer_device_address: draw_data_buffer.device_address,

            accept_mask,

            _pad0: 0,
        }
    }
}
