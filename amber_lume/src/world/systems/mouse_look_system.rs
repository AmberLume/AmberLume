use std::f32::consts::FRAC_PI_2;
use shipyard::{IntoIter, UniqueView, ViewMut};
use crate::input_handler::input_frame::PointerId;
use crate::world::components::camera_component::CameraComponent;
use crate::world::unique::user_input_unique::UserInputUnique;

const PITCH_LIMIT: f32 = FRAC_PI_2 - 0.001;
const MOUSE_POINTER_ID: PointerId = PointerId { id: 0 };

pub fn mouse_look_system(
    user_input_unique: UniqueView<UserInputUnique>,
    mut cameras: ViewMut<CameraComponent>,
) {
    let (delta_x, delta_y) = user_input_unique.input_frame
        .get_pointer_by_id(&MOUSE_POINTER_ID)
        .map(|pointer| pointer.position_delta)
        .unwrap_or((0.0, 0.0));

    if delta_x == 0.0 && delta_y == 0.0 {
        return;
    }

    for camera in (&mut cameras).iter() {
        camera.yaw -= delta_x * camera.sensitivity;
        camera.pitch += delta_y * camera.sensitivity;
        camera.pitch = camera.pitch.clamp(-PITCH_LIMIT, PITCH_LIMIT);
    }
}
