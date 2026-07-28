use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};
use crate::render::render_graph::virtual_buffer::physical_buffer::PhysicalBuffer;

#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone)]
pub struct DrawSortPushConstants {
    pub indirect_source_buffer_device_address: DeviceAddress,
    pub indirect_sorted_buffer_device_address: DeviceAddress,
    pub draw_count_buffer_device_address: DeviceAddress,
    pub draw_data_buffer_device_address: DeviceAddress,
    pub statistics_buffer_device_address: DeviceAddress,
}

impl DrawSortPushConstants {
    pub fn create(
        indirect_source_buffer: &PhysicalBuffer,
        indirect_sorted_buffer: &PhysicalBuffer,
        draw_count_buffer: &PhysicalBuffer,
        draw_data_buffer: &PhysicalBuffer,
        statistics_buffer_device_address: DeviceAddress,
    ) -> Self {
        Self {
            indirect_source_buffer_device_address: indirect_source_buffer.device_address,
            indirect_sorted_buffer_device_address: indirect_sorted_buffer.device_address,
            draw_count_buffer_device_address: draw_count_buffer.device_address,
            draw_data_buffer_device_address: draw_data_buffer.device_address,
            statistics_buffer_device_address,
        }
    }
}
