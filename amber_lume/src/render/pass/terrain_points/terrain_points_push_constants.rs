use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};
use render_graph::PhysicalBuffer;

#[repr(C, align(8))]
#[derive(Pod, Zeroable, Copy, Clone)]
pub struct TerrainPointsPushConstants {
    scene_buffer_device_address: DeviceAddress,
    chunk_buffer_device_address: DeviceAddress,
    vertex_buffer_device_address: DeviceAddress,

    node_count: u32,
    point_size: f32,
    viewport_height: f32,

    _pad0: [u32; 23],
}

impl TerrainPointsPushConstants {
    pub fn create(
        scene_buffer: PhysicalBuffer,
        chunk_buffer: PhysicalBuffer,
        vertex_buffer_device_address: DeviceAddress,
        node_count: u32,
        point_size: f32,
        viewport_height: f32,
    ) -> Self {
        Self {
            scene_buffer_device_address: scene_buffer.device_address,
            chunk_buffer_device_address: chunk_buffer.device_address,
            vertex_buffer_device_address,

            node_count,
            point_size,
            viewport_height,

            _pad0: [0; 23],
        }
    }
}
