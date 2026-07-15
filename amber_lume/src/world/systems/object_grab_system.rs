use glam::Vec3;
use shipyard::{IntoIter, UniqueView, UniqueViewMut, View, ViewMut};
use crate::input_handler::hardware_pointer_key_codes::HardwarePointerKeyCodes;
use crate::physics::body_parameter::BodyParameters;
use crate::physics::object_grab::ObjectGrab;
use crate::world::components::focus_component::FocusComponent;
use crate::world::components::grab_component::GrabComponent;
use crate::world::components::position_component::PositionComponent;
use crate::world::components::rotation_component::RotationComponent;
use crate::world::physics::physics_world_unique::PhysicsWorldUnique;
use crate::world::unique::user_input_unique::UserInputUnique;

pub fn object_grab_system(
    user_input_unique: UniqueView<UserInputUnique>,
    mut physics_world_unique: UniqueViewMut<PhysicsWorldUnique>,
    positions: View<PositionComponent>,
    rotations: View<RotationComponent>,
    focuses: View<FocusComponent>,
    mut grabs: ViewMut<GrabComponent>,
) {
    let Some(pointer) = user_input_unique.input_frame.get_primary_pointer() else {
        return;
    };

    let grab_just_pressed = pointer.key_just_pressed(HardwarePointerKeyCodes::Left);
    let grab_pressed = pointer.key_pressed(HardwarePointerKeyCodes::Left);

    for (position, rotation, focus, grab) in (&positions, &rotations, &focuses, &mut grabs).iter() {
        let origin = position.position;
        let forward = (rotation.rotation * Vec3::Z).normalize();

        if grab_just_pressed && grab.grab.is_none() {
            let Some(hit) = focus.hit else {
                continue;
            };

            if BodyParameters::is_dynamic(&physics_world_unique.handle, hit.body) {
                grab.grab = ObjectGrab::grab(&mut physics_world_unique.handle, hit.body, &grab.params);
            }
        } else if !grab_pressed {
            if let Some(object_grab) = grab.grab.take() {
                object_grab.release(&mut physics_world_unique.handle);
            }
        }

        if let Some(object_grab) = &grab.grab {
            let position = origin + forward * grab.params.distance;

            object_grab.move_anchor(&mut physics_world_unique.handle, position, grab.params);
        }
    }
}
