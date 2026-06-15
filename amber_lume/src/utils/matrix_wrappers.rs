use glam::{Mat4, Vec3, Vec4};

#[derive(Clone, Copy)]
pub struct ViewMatrix {
    pub value: Mat4,
}

impl ViewMatrix {
    pub fn from_mat4(value: Mat4) -> Self {
        Self { value }
    }

    pub fn new(position: Vec3, target: Vec3) -> Self {
        Self {
            value: Mat4::look_at_rh(position, target, Vec3::Y),
        }
    }
}

#[derive(Clone, Copy)]
pub struct ProjectionMatrix {
    pub value: Mat4,
}

impl ProjectionMatrix {
    pub fn from_mat4(value: Mat4) -> Self {
        Self { value }
    }

    pub fn new(near: f32, far: f32, fov: f32, aspect_ratio: f32) -> Self {
        Self {
            value: Mat4::perspective_rh(fov.to_radians(), aspect_ratio, near, far),
        }
    }
}

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
            Vec4::new(0.0, 0.0, 0.5, 0.0),
            Vec4::new(0.0, 0.0, 0.5, 1.0),
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
