use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};
use gpu::BufferRange;

#[repr(C, align(8))]
#[derive(Pod, Zeroable, Copy, Clone)]
pub struct SkinCachePushConstants {
    skin_cache_instance_buffer_device_address: DeviceAddress,
    skinning_instance_buffer_device_address: DeviceAddress,
    entity_buffer_device_address: DeviceAddress,
    entity_motion_buffer_device_address: DeviceAddress,
    bone_transform_buffer_device_address: DeviceAddress,
    mesh_vertex_buffer_device_address: DeviceAddress,
    mesh_vertex_attribute_buffer_device_address: DeviceAddress,
    mesh_vertex_skin_buffer_device_address: DeviceAddress,
    skin_cache_vertex_buffer_device_address: DeviceAddress,
    skin_cache_vertex_attribute_buffer_device_address: DeviceAddress,
    skin_cache_previous_vertex_buffer_device_address: DeviceAddress,

    instance_count: u32,
    vertex_count: u32,

    _pad0: [u32; 8],
}

impl SkinCachePushConstants {
    pub fn create(
        skin_cache_instance_buffer: BufferRange,
        skinning_instance_buffer: BufferRange,
        entity_buffer: BufferRange,
        entity_motion_buffer: BufferRange,
        bone_transform_buffer: BufferRange,
        mesh_vertex_buffer: BufferRange,
        mesh_vertex_attribute_buffer: BufferRange,
        mesh_vertex_skin_buffer: BufferRange,
        skin_cache_vertex_buffer: BufferRange,
        skin_cache_vertex_attribute_buffer: BufferRange,
        skin_cache_previous_vertex_buffer: BufferRange,
        instance_count: u32,
        vertex_count: u32,
    ) -> Self {
        Self {
            skin_cache_instance_buffer_device_address: skin_cache_instance_buffer.device_address,
            skinning_instance_buffer_device_address: skinning_instance_buffer.device_address,
            entity_buffer_device_address: entity_buffer.device_address,
            entity_motion_buffer_device_address: entity_motion_buffer.device_address,
            bone_transform_buffer_device_address: bone_transform_buffer.device_address,
            mesh_vertex_buffer_device_address: mesh_vertex_buffer.device_address,
            mesh_vertex_attribute_buffer_device_address: mesh_vertex_attribute_buffer.device_address,
            mesh_vertex_skin_buffer_device_address: mesh_vertex_skin_buffer.device_address,
            skin_cache_vertex_buffer_device_address: skin_cache_vertex_buffer.device_address,
            skin_cache_vertex_attribute_buffer_device_address: skin_cache_vertex_attribute_buffer.device_address,
            skin_cache_previous_vertex_buffer_device_address: skin_cache_previous_vertex_buffer.device_address,

            instance_count,
            vertex_count,

            _pad0: [0; 8],
        }
    }
}
