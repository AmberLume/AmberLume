use ash::vk::DeviceAddress;
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec4Swizzles};

#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone)]
pub struct FrustumPlanes {
    pub planes: [[f32; 4]; 6],
}

impl FrustumPlanes {
    pub fn from_matrix(matrix: Mat4) -> Self {
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

        Self { planes }
    }
}

#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone)]
pub struct CullingPushConstants {
    pub scene_buffer_device_address: DeviceAddress,
    pub gpu_render_stats_buffer_device_address: DeviceAddress,

    pub frustum_planes: FrustumPlanes,

    pub entity_count: u32,
    _pad0: u32,
}

impl CullingPushConstants {
    pub fn create(
        scene_buffer_device_address: DeviceAddress,
        gpu_render_stats_buffer_device_address: DeviceAddress,
        frustum_planes: FrustumPlanes,
        entity_count: u32,
    ) -> Self {
        Self {
            scene_buffer_device_address,
            gpu_render_stats_buffer_device_address,

            frustum_planes,

            entity_count,
            _pad0: 0,
        }
    }
}
