use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};
use gpu::BufferRange;

#[repr(C, align(8))]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct DepthPushConstants {
    pub scene_buffer_device_address: DeviceAddress,
    pub draw_data_buffer_device_address: DeviceAddress,
    pub entity_buffer_device_address: DeviceAddress,
    pub entity_motion_buffer_device_address: DeviceAddress,
    pub submesh_buffer_device_address: DeviceAddress,
    pub mesh_vertex_buffer_device_address: DeviceAddress,
    pub mesh_vertex_skin_buffer_device_address: DeviceAddress,
    pub bone_transform_buffer_device_address: DeviceAddress,
    
    _pad0: [u32; 16],
}

impl DepthPushConstants {
    pub fn create(
        scene_buffer: BufferRange,
        draw_data_buffer: BufferRange,
        entity_buffer: BufferRange,
        entity_motion_buffer: BufferRange,
        submesh_buffer: BufferRange,
        mesh_vertex_buffer: BufferRange,
        mesh_vertex_skin_buffer: BufferRange,
        bone_transform_buffer: BufferRange,
    ) -> Self {
        Self {
            scene_buffer_device_address: scene_buffer.device_address,
            draw_data_buffer_device_address: draw_data_buffer.device_address,
            entity_buffer_device_address: entity_buffer.device_address,
            entity_motion_buffer_device_address: entity_motion_buffer.device_address,
            submesh_buffer_device_address: submesh_buffer.device_address,
            mesh_vertex_buffer_device_address: mesh_vertex_buffer.device_address,
            mesh_vertex_skin_buffer_device_address: mesh_vertex_skin_buffer.device_address,
            bone_transform_buffer_device_address: bone_transform_buffer.device_address,
            
            _pad0: [0; 16],
        }
    }
}
