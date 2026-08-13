use render_graph::PhysicalReadback;
use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};
use render_graph::PhysicalBuffer;

#[repr(C, align(8))]
#[derive(Pod, Zeroable, Copy, Clone)]
pub struct CullingIndirectPushConstants {
    pub culling_views_buffer_device_address: DeviceAddress,
    pub entity_buffer_device_address: DeviceAddress,
    pub mesh_buffer_device_address: DeviceAddress,
    pub submesh_buffer_device_address: DeviceAddress,
    pub meta_statistics_buffer_device_address: DeviceAddress,

    pub cull_requests_buffer_device_address: DeviceAddress,
    pub indirect_buffer_device_address: DeviceAddress,
    pub draw_count_buffer_device_address: DeviceAddress,
    pub draw_data_buffer_device_address: DeviceAddress,
    pub material_buffer_device_address: DeviceAddress,
    pub scene_buffer_device_address: DeviceAddress,

    pub view_count: u32,
    pub entity_count: u32,
    pub combine_views: u32,
    pub request_count: u32,

    _pad0: [u32; 6],
}

impl CullingIndirectPushConstants {
    pub fn create(
        culling_views_buffer: PhysicalBuffer,
        entity_buffer: PhysicalBuffer,
        mesh_buffer_device_address: DeviceAddress,
        submesh_buffer_device_address: DeviceAddress,
        statistics: PhysicalReadback,
        cull_requests_buffer: PhysicalBuffer,
        indirect_buffer: PhysicalBuffer,
        draw_count_buffer: PhysicalBuffer,
        draw_data_buffer: PhysicalBuffer,
        material_buffer_device_address: DeviceAddress,
        scene_buffer: PhysicalBuffer,
        view_count: u32,
        entity_count: u32,
        combine_views: bool,
        request_count: u32,
    ) -> Self {
        Self {
            culling_views_buffer_device_address: culling_views_buffer.device_address,
            entity_buffer_device_address: entity_buffer.device_address,
            mesh_buffer_device_address,
            submesh_buffer_device_address,
            meta_statistics_buffer_device_address: statistics.device_address,

            cull_requests_buffer_device_address: cull_requests_buffer.device_address,
            indirect_buffer_device_address: indirect_buffer.device_address,
            draw_count_buffer_device_address: draw_count_buffer.device_address,
            draw_data_buffer_device_address: draw_data_buffer.device_address,
            material_buffer_device_address,
            scene_buffer_device_address: scene_buffer.device_address,

            view_count,
            entity_count,
            combine_views: combine_views as u32,
            request_count,

            _pad0: [0; 6],
        }
    }
}
