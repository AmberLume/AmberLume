use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};
use render_graph::PhysicalBuffer;

#[repr(C, align(8))]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct DepthPushConstants {
    pub scene_buffer_device_address: DeviceAddress,
    pub draw_data_buffer_device_address: DeviceAddress,
    pub entity_buffer_device_address: DeviceAddress,
    pub vertex_buffer_device_address: DeviceAddress,
    pub bone_transform_buffer_device_address: DeviceAddress,
    
    _pad0: [u32; 22],
}

impl DepthPushConstants {
    pub fn create(
        scene_buffer: PhysicalBuffer,
        draw_data_buffer: PhysicalBuffer,
        entity_buffer: PhysicalBuffer,
        vertex_buffer_device_address: DeviceAddress,
        bone_transform_buffer: PhysicalBuffer,
    ) -> Self {
        Self {
            scene_buffer_device_address: scene_buffer.device_address,
            draw_data_buffer_device_address: draw_data_buffer.device_address,
            entity_buffer_device_address: entity_buffer.device_address,
            vertex_buffer_device_address,
            bone_transform_buffer_device_address: bone_transform_buffer.device_address,
            
            _pad0: [0; 22],
        }
    }
}
