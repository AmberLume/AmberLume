use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec4Swizzles};

#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone)]
pub struct CullingIndirectPushConstants {
    pub frustum_planes: [[f32; 4]; 6],

    pub indirect_buffer_device_address: DeviceAddress,

    pub entity_buffer_device_address: DeviceAddress,

    pub draw_data_buffer_device_address: DeviceAddress,
    pub draw_count_buffer_device_address: DeviceAddress,

    pub submesh_buffer_device_address: DeviceAddress,
    pub model_buffer_device_address: DeviceAddress,

    pub gpu_render_stats_buffer_device_address: DeviceAddress,

    pub entity_count: u32,

    _pad0: u32,
}

impl CullingIndirectPushConstants {
    pub fn create(
        projection_matrix: Mat4,
        indirect_buffer_device_address: DeviceAddress,
        entity_buffer_device_address: DeviceAddress,
        draw_data_buffer_device_address: DeviceAddress,
        draw_count_buffer_device_address: DeviceAddress,
        submesh_buffer_device_address: DeviceAddress,
        model_buffer_device_address: DeviceAddress,
        gpu_render_stats_buffer_device_address: DeviceAddress,
        entity_count: u32,
    ) -> Self {
        Self {
            frustum_planes: Self::frustum_planes_from_matrix(projection_matrix),

            indirect_buffer_device_address,

            entity_buffer_device_address,

            draw_data_buffer_device_address,
            draw_count_buffer_device_address,

            submesh_buffer_device_address,
            model_buffer_device_address,

            gpu_render_stats_buffer_device_address,

            entity_count,

            _pad0: 0,
        }
    }

    pub fn frustum_planes_from_matrix(matrix: Mat4) -> [[f32; 4]; 6] {
        let mut planes = [[0.0f32; 4]; 6];

        let combinations = [
            matrix.row(3) + matrix.row(0),
            matrix.row(3) - matrix.row(0),
            matrix.row(3) + matrix.row(1),
            matrix.row(3) - matrix.row(1),
            matrix.row(2),
            matrix.row(3) - matrix.row(2),
        ];

        for (index, plane) in combinations.iter().enumerate() {
            let length = plane.xyz().length();

            let normalized = plane / length;

            planes[index] = normalized.to_array();
        }

        planes
    }
}
