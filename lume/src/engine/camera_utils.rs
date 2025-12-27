use glam::{Mat4, Vec3};

pub fn create_view_projection_matrix(
    aspect_ratio: f32,
    fov: f32,
    look_at: Vec3,
    distance: f32,
    near: f32,
    far: f32,
) -> Mat4 {
    let camera_position = look_at + Vec3::new(0.0, -distance * 0.5, distance);
    let view = Mat4::look_at_rh(camera_position, look_at, Vec3::Y);

    let field_of_view_radians = fov.to_radians();
    let mut projection = Mat4::perspective_rh(field_of_view_radians, aspect_ratio, near, far);

    projection.y_axis.y *= -1.0;

    projection * view
}
