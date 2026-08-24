use gpu::BufferRange;
use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};

#[repr(C, align(8))]
#[derive(Pod, Zeroable, Copy, Clone)]
pub struct TerrainStitchPushConstants {
    request_buffer_device_address: DeviceAddress,
    edge_height_buffer_device_address: DeviceAddress,
    vertex_buffer_device_address: DeviceAddress,
    mesh_buffer_device_address: DeviceAddress,
    submesh_buffer_device_address: DeviceAddress,

    node_count: u32,
    nodes: u32,

    _pad0: [u32; 20],
}

impl TerrainStitchPushConstants {
    pub fn create(
        request_buffer: BufferRange,
        edge_height_buffer: BufferRange,
        vertex_buffer: BufferRange,
        mesh_buffer: BufferRange,
        submesh_buffer: BufferRange,
        node_count: u32,
        nodes: u32,
    ) -> Self {
        Self {
            request_buffer_device_address: request_buffer.device_address,
            edge_height_buffer_device_address: edge_height_buffer.device_address,
            vertex_buffer_device_address: vertex_buffer.device_address,
            mesh_buffer_device_address: mesh_buffer.device_address,
            submesh_buffer_device_address: submesh_buffer.device_address,

            node_count,
            nodes,

            _pad0: [0; 20],
        }
    }
}
