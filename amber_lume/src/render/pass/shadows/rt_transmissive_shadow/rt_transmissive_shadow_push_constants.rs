use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};
use index_allocator::ResourceId;
use gpu::BufferRange;

#[repr(C, align(8))]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct RTTransmissiveShadowPushConstants {
    pub scene_buffer_device_address: DeviceAddress,
    pub entity_buffer_device_address: DeviceAddress,
    pub mesh_buffer_device_address: DeviceAddress,
    pub submesh_buffer_device_address: DeviceAddress,
    pub material_buffer_device_address: DeviceAddress,

    pub depth_descriptor_id: u32,
    pub normal_descriptor_id: u32,
    pub transmittance_storage_id: u32,
    pub tlas_descriptor_id: u32,

    pub sun_angular_radius: f32,
    pub sample_count: u32,
    pub frame_number: u32,

    _pad0: u32,
}

impl RTTransmissiveShadowPushConstants {
    pub fn create(
        scene_buffer: BufferRange,
        entity_buffer: BufferRange,
        mesh_buffer: BufferRange,
        submesh_buffer: BufferRange,
        material_buffer: BufferRange,
        depth_descriptor_id: ResourceId,
        normal_descriptor_id: ResourceId,
        transmittance_storage_id: ResourceId,
        tlas_descriptor_id: ResourceId,
        sun_angular_radius: f32,
        sample_count: u32,
        frame_number: u32,
    ) -> Self {
        Self {
            scene_buffer_device_address: scene_buffer.device_address,
            entity_buffer_device_address: entity_buffer.device_address,
            mesh_buffer_device_address: mesh_buffer.device_address,
            submesh_buffer_device_address: submesh_buffer.device_address,
            material_buffer_device_address: material_buffer.device_address,

            depth_descriptor_id: depth_descriptor_id.inner,
            normal_descriptor_id: normal_descriptor_id.inner,
            transmittance_storage_id: transmittance_storage_id.inner,
            tlas_descriptor_id: tlas_descriptor_id.inner,

            sun_angular_radius,
            sample_count,
            frame_number,

            _pad0: 0,
        }
    }
}
