use std::f32::consts::FRAC_PI_2;
use glam::{EulerRot, Quat};
use input::HardwarePointerKeyCodes;
use shipyard::{IntoIter, UniqueViewMut, View, ViewMut};
use crate::world::components::camera_component::CameraComponent;
use crate::world::components::rotation_component::RotationComponent;
use crate::world::unique::user_input_unique::UserInputUnique;

const PITCH_LIMIT: f32 = FRAC_PI_2 - 0.001;
const SENSITIVITY: f32 = 0.001;

pub fn mouse_look_system(
    mut user_input_unique: UniqueViewMut<UserInputUnique>,
    cameras: View<CameraComponent>,
    mut rotations: ViewMut<RotationComponent>,
) {
    let Some(input) = user_input_unique.input.as_mut() else {
        return;
    };

    let orbit_button = input.button(HardwarePointerKeyCodes::Right, true).is_down();

    let Some(motion) = input.motion(true) else {
        return;
    };

    if motion.x == 0.0 && motion.y == 0.0 {
        return;
    }

    for (rotation, camera) in (&mut rotations, &cameras).iter() {
        if camera.target_id.is_some() && !orbit_button {
            continue;
        }

        let (yaw, pitch, _) = rotation.rotation.to_euler(EulerRot::YXZ);

        let yaw = yaw - motion.x * SENSITIVITY;
        let pitch = (pitch + motion.y * SENSITIVITY).clamp(-PITCH_LIMIT, PITCH_LIMIT);

        rotation.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0);
    }
}
