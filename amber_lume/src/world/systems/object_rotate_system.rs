use glam::{EulerRot, Quat};
use shipyard::{IntoIter, UniqueViewMut, ViewMut};
use crate::input_handler::hardware_pointer_key_codes::HardwarePointerKeyCodes;
use crate::world::components::grab_component::GrabComponent;
use crate::world::unique::user_input_unique::UserInputUnique;

const SENSITIVITY: f32 = 0.005;

pub fn object_rotate_system(
    mut user_input_unique: UniqueViewMut<UserInputUnique>,
    mut grabs: ViewMut<GrabComponent>,
) {
    let Some(pointer) = user_input_unique.input_frame.get_primary_pointer_mut() else {
        return;
    };

    if !pointer.key_pressed(HardwarePointerKeyCodes::Right) {
        return;
    }

    let (delta_x, delta_y) = pointer.position_delta;
    if delta_x == 0.0 && delta_y == 0.0 {
        return;
    }

    let delta_rotation = Quat::from_euler(
        EulerRot::XYZ,
        -delta_y * SENSITIVITY,
        delta_x * SENSITIVITY,
        0.0,
    );

    let mut consumed = false;
    for grab in (&mut grabs).iter() {
        if let Some(object_grab) = &mut grab.grab {
            object_grab.rotate(delta_rotation);

            consumed = true;
        }
    }

    if consumed {
        pointer.position_delta = (0.0, 0.0);
    }
}
