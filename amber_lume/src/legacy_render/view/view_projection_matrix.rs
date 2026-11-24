use math::{Mat4, Vec4};
use crate::legacy_render::view::projection_matrix::ProjectionMatrix;
use crate::legacy_render::view::view_matrix::ViewMatrix;

#[derive(Clone, Copy)]
pub struct ViewProjectionMatrix {
    pub value: Mat4,
}

impl ViewProjectionMatrix {
    pub fn from_view_projection(view: &ViewMatrix, projection: &ProjectionMatrix) -> Self {
        Self {
            value: projection.value * view.value,
        }
    }

    pub fn vulkan_corrected(self) -> Self {
        let correction = Mat4::from_cols(
            Vec4::new(1.0, 0.0, 0.0, 0.0),
            Vec4::new(0.0, -1.0, 0.0, 0.0),
            Vec4::new(0.0, 0.0, 1.0, 0.0),
            Vec4::new(0.0, 0.0, 0.0, 1.0),
        );

        Self {
            value: correction * self.value,
        }
    }

    pub fn inverted(&self) -> Self {
        Self {
            value: self.value.inverse(),
        }
    }
}
