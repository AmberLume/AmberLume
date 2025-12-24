use ash::vk::Extent2D;
use glam::{Mat4, Vec3};

pub fn create_perspective_view_projection(
    field_of_view: f32,
    extent: &Extent2D,
    distance: f32,
    center: Vec3,
) -> Mat4 {
    let aspect_ratio = extent.width as f32 / extent.height as f32;

    let camera_position = center + Vec3::new(0.0, distance * 0.707, distance * 0.707);
    let view = Mat4::look_at_rh(camera_position, center, Vec3::Y);

    let field_of_view_radians = field_of_view.to_radians();
    let near = 0.1;
    let far = distance * 2.0;

    let mut projection = Mat4::perspective_rh(field_of_view_radians, aspect_ratio, near, far);

    projection.y_axis.y *= -1.0;

    projection * view
}
