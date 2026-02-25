use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};
use crate::resources::dynamic::resource_provider::ResourceId;

#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone)]
pub struct ShadowMaskPushConstants {
    pub shadow_buffer_device_address: DeviceAddress,
    
    pub cascade_count: u32,
    pub camera_near: f32,
    pub camera_far: f32,
    pub bias: f32,
    pub pcf_radius: i32,

    pub depth_descriptor_id: u32,
    pub global_shadow_descriptor_id: u32,
    
    _pad0: u32,
}

impl ShadowMaskPushConstants {
    pub fn create(
        shadow_buffer_device_address: DeviceAddress,
        cascade_count: u32,
        camera_near: f32,
        camera_far: f32,
        bias: f32,
        pcf_radius: i32,
        depth_descriptor_id: ResourceId,
        global_shadow_descriptor_id: ResourceId,
    ) -> Self {
        Self {
            shadow_buffer_device_address,
            
            cascade_count,
            camera_near,
            camera_far,
            bias,
            pcf_radius,

            depth_descriptor_id,
            global_shadow_descriptor_id,
            
            _pad0: 0,
        }
    }
}
