use glam::Mat4;

#[derive(Clone, Copy)]
pub struct ProjectionMatrix {
    pub value: Mat4,
}

impl ProjectionMatrix {
    pub fn new(near: f32, far: f32, fov: f32, aspect_ratio: f32) -> Self {
        Self {
            value: Mat4::perspective_rh(fov.to_radians(), aspect_ratio, far, near),
        }
    }
}
