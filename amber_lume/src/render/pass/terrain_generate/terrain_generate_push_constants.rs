use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};
use gpu::BufferRange;

#[repr(C, align(8))]
#[derive(Pod, Zeroable, Copy, Clone)]
pub struct TerrainGeneratePushConstants {
    request_buffer_device_address: DeviceAddress,
    height_buffer_device_address: DeviceAddress,
    mesh_vertex_buffer_device_address: DeviceAddress,
    mesh_vertex_attribute_buffer_device_address: DeviceAddress,
    mesh_buffer_device_address: DeviceAddress,
    submesh_buffer_device_address: DeviceAddress,

    node_count: u32,
    nodes: u32,
    window_stride: u32,

    _pad0: [u32; 17],
}

impl TerrainGeneratePushConstants {
    pub fn create(
        request_buffer: BufferRange,
        height_buffer: BufferRange,
        mesh_vertex_buffer: BufferRange,
        mesh_vertex_attribute_buffer: BufferRange,
        mesh_buffer: BufferRange,
        submesh_buffer: BufferRange,
        node_count: u32,
        nodes: u32,
        window_stride: u32,
    ) -> Self {
        Self {
            request_buffer_device_address: request_buffer.device_address,
            height_buffer_device_address: height_buffer.device_address,
            mesh_vertex_buffer_device_address: mesh_vertex_buffer.device_address,
            mesh_vertex_attribute_buffer_device_address: mesh_vertex_attribute_buffer.device_address,
            mesh_buffer_device_address: mesh_buffer.device_address,
            submesh_buffer_device_address: submesh_buffer.device_address,

            node_count,
            nodes,
            window_stride,

            _pad0: [0; 17],
        }
    }
}
