use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct ColliderCullingPushConstants {
    pub scene_buffer_device_address: DeviceAddress,
    
    pub collider_count: u32,
    _pad0: u32,
}

impl ColliderCullingPushConstants {
    pub fn create(
        scene_buffer_device_address: DeviceAddress,
        collider_count: u32,
    ) -> Self {
        Self {
            scene_buffer_device_address,
            
            collider_count,
            _pad0: 0,
        }
    }
}
