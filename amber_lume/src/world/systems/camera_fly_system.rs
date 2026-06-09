use glam::Vec3;
use shipyard::{IntoIter, UniqueView, ViewMut};
use crate::input_handler::hardware_key_codes::HardwareKeyCode;
use crate::world::components::camera_component::{CameraComponent, CameraMode};
use crate::world::unique::render_view_unique::RenderViewUnique;
use crate::world::unique::user_input_unique::UserInputUnique;
use crate::world::unique::world_time_unique::WorldTimeUnique;

pub fn camera_fly_system(
    user_input_unique: UniqueView<UserInputUnique>,
    world_time_unique: UniqueView<WorldTimeUnique>,
    render_view_unique: UniqueView<RenderViewUnique>,
    mut cameras: ViewMut<CameraComponent>,
) {
    let input_frame = &user_input_unique.input_frame;
    let toggle = input_frame.just_pressed(HardwareKeyCode::F1);

    for camera in (&mut cameras).iter() {
        if toggle {
            camera.mode = match camera.mode {
                CameraMode::Attached => {
                    camera.free_position = render_view_unique.resolved_camera.position;

                    CameraMode::Free
                }
                CameraMode::Free => CameraMode::Attached,
            };
        }

        if camera.mode != CameraMode::Free {
            continue;
        }

        let forward = camera.local_rotation() * Vec3::Z;
        let right = forward.cross(Vec3::Y).normalize_or_zero();

        let mut direction = Vec3::ZERO;

        if input_frame.is_down(HardwareKeyCode::W) {
            direction += forward;
        }

        if input_frame.is_down(HardwareKeyCode::S) {
            direction -= forward;
        }

        if input_frame.is_down(HardwareKeyCode::D) {
            direction += right;
        }

        if input_frame.is_down(HardwareKeyCode::A) {
            direction -= right;
        }

        if input_frame.is_down(HardwareKeyCode::Space) {
            direction += Vec3::Y;
        }

        if input_frame.is_down(HardwareKeyCode::C) {
            direction -= Vec3::Y;
        }

        let speed_multiplier = if input_frame.is_down(HardwareKeyCode::Shift) { 2.0 } else { 1.0 };

        camera.free_position += direction.normalize_or_zero() * camera.fly_speed * speed_multiplier * world_time_unique.delta;
    }
}
