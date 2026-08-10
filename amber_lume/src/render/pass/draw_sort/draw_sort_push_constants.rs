use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};
use render_graph::DrawBucket;
use render_graph::PhysicalBuffer;

#[repr(C, align(8))]
#[derive(Pod, Zeroable, Copy, Clone)]
pub struct DrawSortPushConstants {
    pub indirect_source_buffer_device_address: DeviceAddress,
    pub indirect_sorted_buffer_device_address: DeviceAddress,
    pub draw_count_buffer_device_address: DeviceAddress,
    pub draw_data_buffer_device_address: DeviceAddress,
    pub statistics_buffer_device_address: DeviceAddress,

    pub count_index: u32,
    pub source_offset: u32,
    pub target_offset: u32,
    pub capacity: u32,
}

impl DrawSortPushConstants {
    pub fn create(
        indirect_buffer: &PhysicalBuffer,
        draw_count_buffer: &PhysicalBuffer,
        draw_data_buffer: &PhysicalBuffer,
        statistics_buffer_device_address: DeviceAddress,
        source_bucket: DrawBucket,
        sorted_bucket: DrawBucket,
    ) -> Self {
        Self {
            indirect_source_buffer_device_address: indirect_buffer.device_address,
            indirect_sorted_buffer_device_address: indirect_buffer.device_address,
            draw_count_buffer_device_address: draw_count_buffer.device_address,
            draw_data_buffer_device_address: draw_data_buffer.device_address,
            statistics_buffer_device_address,

            count_index: source_bucket.count_index,
            source_offset: source_bucket.draw_offset,
            target_offset: sorted_bucket.draw_offset,
            capacity: sorted_bucket.capacity,
        }
    }
}
