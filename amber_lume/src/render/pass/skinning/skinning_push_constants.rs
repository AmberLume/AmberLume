use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};
use gpu::BufferRange;

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
       skinning_instance_buffer: BufferRange,
       animation_buffer: BufferRange,
       animation_frame_buffer: BufferRange,
       skeleton_buffer: BufferRange,
       skeleton_bone_buffer: BufferRange,
       bone_transform_buffer: BufferRange,
       instance_count: u32,
    ) -> Self {
        Self {
            skinning_instance_buffer_device_address: skinning_instance_buffer.device_address,
            animation_buffer_device_address: animation_buffer.device_address,
            animation_frame_buffer_device_address: animation_frame_buffer.device_address,
            skeleton_buffer_device_address: skeleton_buffer.device_address,
            skeleton_bone_buffer_device_address: skeleton_bone_buffer.device_address,
            bone_transform_buffer_device_address: bone_transform_buffer.device_address,
            
            instance_count,
            
            _pad0: [0; 19],
        }
    }
}
