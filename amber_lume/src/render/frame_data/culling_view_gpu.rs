use bytemuck::{Pod, Zeroable};
use glam::Vec4Swizzles;
use gpu::ViewProjectionMatrix;

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct CullingViewGPU {
    pub frustum_planes: [[f32; 4]; 6],
}

impl CullingViewGPU {
    pub fn create(view_projection: &ViewProjectionMatrix) -> Self {
        Self {
            frustum_planes: Self::frustum_planes_from_matrix(view_projection),
        }
    }

    fn frustum_planes_from_matrix(view_projection: &ViewProjectionMatrix) -> [[f32; 4]; 6] {
        let mut planes = [[0.0f32; 4]; 6];

        let combinations = [
            view_projection.value.row(3) + view_projection.value.row(0),
            view_projection.value.row(3) - view_projection.value.row(0),
            view_projection.value.row(3) + view_projection.value.row(1),
            view_projection.value.row(3) - view_projection.value.row(1),
            view_projection.value.row(2),
            view_projection.value.row(3) - view_projection.value.row(2),
        ];

        for (index, plane) in combinations.iter().enumerate() {
            let length = plane.xyz().length();

            let normalized = plane / length;

            planes[index] = normalized.to_array();
        }

        planes
    }
}
