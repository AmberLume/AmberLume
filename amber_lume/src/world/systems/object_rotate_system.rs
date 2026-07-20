use glam::{EulerRot, Quat};
use input::HardwarePointerKeyCodes;
use shipyard::{IntoIter, UniqueViewMut, ViewMut};
use crate::world::components::grab_component::GrabComponent;
use crate::world::unique::user_input_unique::UserInputUnique;

const SENSITIVITY: f32 = 0.005;

pub fn object_rotate_system(
    mut user_input_unique: UniqueViewMut<UserInputUnique>,
    mut grabs: ViewMut<GrabComponent>,
) {
    let Some(input) = user_input_unique.input.as_mut() else {
        return;
    };

    if !(&grabs).iter().any(|grab| grab.grab.is_some()) {
        return;
    }

    if !input.button(HardwarePointerKeyCodes::Right, false).is_down() {
        return;
    }

    let Some(motion) = input.motion(true) else {
        return;
    };

    if motion.x == 0.0 && motion.y == 0.0 {
        return;
    }

    let delta_rotation = Quat::from_euler(
        EulerRot::XYZ,
        -motion.y * SENSITIVITY,
        motion.x * SENSITIVITY,
        0.0,
    );

    for grab in (&mut grabs).iter() {
        if let Some(object_grab) = &mut grab.grab {
            object_grab.rotate(delta_rotation);
        }
    }
}
