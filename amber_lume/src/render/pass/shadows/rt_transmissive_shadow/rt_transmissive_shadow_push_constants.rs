use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};
use crate::render::render_graph::virtual_buffer::physical_buffer::PhysicalBuffer;
use index_allocator::ResourceId;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct RTTransmissiveShadowPushConstants {
    pub sun_direction: [f32; 4],

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
        sun_direction: [f32; 3],
        scene_buffer: PhysicalBuffer,
        entity_buffer: PhysicalBuffer,
        mesh_buffer_device_address: DeviceAddress,
        submesh_buffer_device_address: DeviceAddress,
        material_buffer_device_address: DeviceAddress,
        depth_descriptor_id: ResourceId,
        normal_descriptor_id: ResourceId,
        transmittance_storage_id: ResourceId,
        tlas_descriptor_id: ResourceId,
        sun_angular_radius: f32,
        sample_count: u32,
        frame_number: u32,
    ) -> Self {
        Self {
            sun_direction: [sun_direction[0], sun_direction[1], sun_direction[2], 0.0],

            scene_buffer_device_address: scene_buffer.device_address,
            entity_buffer_device_address: entity_buffer.device_address,
            mesh_buffer_device_address,
            submesh_buffer_device_address,
            material_buffer_device_address,

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
