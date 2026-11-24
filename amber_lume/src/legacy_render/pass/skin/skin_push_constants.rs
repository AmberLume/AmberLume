use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};
use gpu::BufferRange;

#[repr(C, align(8))]
#[derive(Pod, Zeroable, Copy, Clone)]
pub struct SkinPushConstants {
    skin_target_buffer_device_address: DeviceAddress,
    skinning_instance_buffer_device_address: DeviceAddress,
    bone_transform_buffer_device_address: DeviceAddress,
    vertex_position_buffer_device_address: DeviceAddress,
    vertex_normal_tangent_buffer_device_address: DeviceAddress,
    vertex_skin_buffer_device_address: DeviceAddress,

    target_count: u32,
    vertex_count: u32,

    _pad0: [u32; 18],
}

impl SkinPushConstants {
    pub fn create(
        skin_target_buffer: BufferRange,
        skinning_instance_buffer: BufferRange,
        bone_transform_buffer: BufferRange,
        vertex_position_buffer: BufferRange,
        vertex_normal_tangent_buffer: BufferRange,
        vertex_skin_buffer: BufferRange,
        target_count: u32,
        vertex_count: u32,
    ) -> Self {
        Self {
            skin_target_buffer_device_address: skin_target_buffer.device_address,
            skinning_instance_buffer_device_address: skinning_instance_buffer.device_address,
            bone_transform_buffer_device_address: bone_transform_buffer.device_address,
            vertex_position_buffer_device_address: vertex_position_buffer.device_address,
            vertex_normal_tangent_buffer_device_address: vertex_normal_tangent_buffer.device_address,
            vertex_skin_buffer_device_address: vertex_skin_buffer.device_address,

            target_count,
            vertex_count,

            _pad0: [0; 18],
        }
    }
}
