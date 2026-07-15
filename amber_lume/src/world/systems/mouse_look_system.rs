use std::f32::consts::FRAC_PI_2;
use glam::{EulerRot, Quat};
use shipyard::{IntoIter, UniqueView, View, ViewMut};
use crate::input_handler::input_frame::PointerId;
use crate::world::components::camera_component::CameraComponent;
use crate::world::components::rotation_component::RotationComponent;
use crate::world::unique::user_input_unique::UserInputUnique;

const PITCH_LIMIT: f32 = FRAC_PI_2 - 0.001;
const MOUSE_POINTER_ID: PointerId = PointerId { id: 0 };
const SENSITIVITY: f32 = 0.001;

pub fn mouse_look_system(
    user_input_unique: UniqueView<UserInputUnique>,
    cameras: View<CameraComponent>,
    mut rotations: ViewMut<RotationComponent>,
) {
    let (delta_x, delta_y) = user_input_unique.input_frame
        .get_pointer_by_id(&MOUSE_POINTER_ID)
        .map(|pointer| pointer.position_delta)
        .unwrap_or((0.0, 0.0));

    if delta_x == 0.0 && delta_y == 0.0 {
        return;
    }

    for (rotation, _camera) in (&mut rotations, &cameras).iter() {
        let (yaw, pitch, _) = rotation.rotation.to_euler(EulerRot::YXZ);

        let yaw = yaw - delta_x * SENSITIVITY;
        let pitch = (pitch + delta_y * SENSITIVITY).clamp(-PITCH_LIMIT, PITCH_LIMIT);

        rotation.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0);
    }
}
