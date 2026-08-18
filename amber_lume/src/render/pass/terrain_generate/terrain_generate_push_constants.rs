use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};
use render_graph::PhysicalBuffer;

#[repr(C, align(8))]
#[derive(Pod, Zeroable, Copy, Clone)]
pub struct TerrainGeneratePushConstants {
    request_buffer_device_address: DeviceAddress,
    height_buffer_device_address: DeviceAddress,
    vertex_buffer_device_address: DeviceAddress,

    node_count: u32,
    nodes: u32,
    window_stride: u32,

    _pad0: [u32; 23],
}

impl TerrainGeneratePushConstants {
    pub fn create(
        request_buffer: PhysicalBuffer,
        height_buffer: PhysicalBuffer,
        vertex_buffer_device_address: DeviceAddress,
        node_count: u32,
        nodes: u32,
        window_stride: u32,
    ) -> Self {
        Self {
            request_buffer_device_address: request_buffer.device_address,
            height_buffer_device_address: height_buffer.device_address,
            vertex_buffer_device_address,

            node_count,
            nodes,
            window_stride,

            _pad0: [0; 23],
        }
    }
}
