use bytemuck::{Pod, Zeroable};
use crate::legacy_render::view::view_projection_matrix::ViewProjectionMatrix;
use math::Vec4;

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct CullingViewGPU {
    pub frustum_planes: [Vec4; 6],
}

impl CullingViewGPU {
    pub fn create(view_projection: &ViewProjectionMatrix) -> Self {
        let mut frustum_planes = [Vec4::ZERO; 6];

        let combinations = [
            view_projection.value.row(3) + view_projection.value.row(0),
            view_projection.value.row(3) - view_projection.value.row(0),
            view_projection.value.row(3) + view_projection.value.row(1),
            view_projection.value.row(3) - view_projection.value.row(1),
            view_projection.value.row(2),
            view_projection.value.row(3) - view_projection.value.row(2),
        ];

        for (index, plane) in combinations.iter().enumerate() {
            let length = plane.truncate().length();

            let normalized = *plane / length;

            frustum_planes[index] = normalized;
        }

        Self {
            frustum_planes,
        }
    }
}
