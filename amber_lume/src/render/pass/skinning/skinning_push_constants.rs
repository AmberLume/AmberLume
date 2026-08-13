use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};
use render_graph::PhysicalBuffer;

#[repr(C, align(8))]
#[derive(Pod, Zeroable, Copy, Clone)]
pub struct SkinningPushConstants {
    skinning_instance_buffer_device_address: DeviceAddress,
    animation_buffer_device_address: DeviceAddress,
    animation_frame_buffer_device_address: DeviceAddress,
    skeleton_buffer_device_address: DeviceAddress,
    skeleton_bone_buffer_device_address: DeviceAddress,
    bone_transform_buffer_device_address: DeviceAddress,
    
    instance_count: u32,
    
    _pad0: [u32; 19],
}

impl SkinningPushConstants {
    pub fn create(
       skinning_instance_buffer_device_address: DeviceAddress,
       animation_buffer_device_address: DeviceAddress,
       animation_frame_buffer_device_address: DeviceAddress,
       skeleton_buffer_device_address: DeviceAddress,
       skeleton_bone_buffer_device_address: DeviceAddress,
       bone_transform_buffer: PhysicalBuffer,
       instance_count: u32,
    ) -> Self {
        Self {
            skinning_instance_buffer_device_address,
            animation_buffer_device_address,
            animation_frame_buffer_device_address,
            skeleton_buffer_device_address,
            skeleton_bone_buffer_device_address,
            bone_transform_buffer_device_address: bone_transform_buffer.device_address,
            
            instance_count,
            
            _pad0: [0; 19],
        }
    }
}
