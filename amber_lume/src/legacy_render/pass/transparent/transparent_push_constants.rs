use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};
use gpu::BufferRange;

#[repr(C, align(8))]
#[derive(Pod, Zeroable, Copy, Clone)]
pub struct TransparentPushConstants {
    pub camera_buffer_device_address: DeviceAddress,
    pub draw_data_buffer_device_address: DeviceAddress,
    pub entity_buffer_device_address: DeviceAddress,
    pub submesh_buffer_device_address: DeviceAddress,
    pub vertex_position_buffer_device_address: DeviceAddress,
    pub vertex_normal_tangent_buffer_device_address: DeviceAddress,
    pub vertex_uv_buffer_device_address: DeviceAddress,

    pub scene_buffer_device_address: DeviceAddress,
    pub material_buffer_device_address: DeviceAddress,

    pub sh_descriptor_id: u32,
    pub brdf_lut_descriptor_id: u32,

    _pad0: [u32; 12],
}

impl TransparentPushConstants {
    pub fn create(
        camera_buffer: BufferRange,
        draw_data_buffer: BufferRange,
        entity_buffer: BufferRange,
        submesh_buffer: BufferRange,
        vertex_position_buffer: BufferRange,
        vertex_normal_tangent_buffer: BufferRange,
        vertex_uv_buffer: BufferRange,
        scene_buffer: BufferRange,
        material_buffer: BufferRange,
        sh_descriptor_id: u32,
        brdf_lut_descriptor_id: u32,
    ) -> Self {
        Self {
            camera_buffer_device_address: camera_buffer.device_address,
            draw_data_buffer_device_address: draw_data_buffer.device_address,
            entity_buffer_device_address: entity_buffer.device_address,
            submesh_buffer_device_address: submesh_buffer.device_address,
            vertex_position_buffer_device_address: vertex_position_buffer.device_address,
            vertex_normal_tangent_buffer_device_address: vertex_normal_tangent_buffer.device_address,
            vertex_uv_buffer_device_address: vertex_uv_buffer.device_address,

            scene_buffer_device_address: scene_buffer.device_address,
            material_buffer_device_address: material_buffer.device_address,

            sh_descriptor_id,
            brdf_lut_descriptor_id,

            _pad0: [0; 12],
        }
    }
}
