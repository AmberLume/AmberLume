#[repr(C)]
pub struct Camera {
    pub view: [[f32; 4]; 4],
    pub projection: [[f32; 4]; 4],
    pub view_projection: [[f32; 4]; 4],
    pub position: [[f32; 3]; 3],
}
