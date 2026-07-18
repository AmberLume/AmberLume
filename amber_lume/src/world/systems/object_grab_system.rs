use glam::Vec3;
use physics::GrabConfig;
use shipyard::{IntoIter, UniqueView, UniqueViewMut, View, ViewMut};
use crate::input_handler::hardware_pointer_key_codes::HardwarePointerKeyCodes;
use crate::world::components::focus_component::FocusComponent;
use crate::world::components::grab_component::GrabComponent;
use crate::world::components::position_component::PositionComponent;
use crate::world::components::rotation_component::RotationComponent;
use crate::world::physics::physics_context_unique::PhysicsContextUnique;
use crate::world::unique::user_input_unique::UserInputUnique;

pub fn object_grab_system(
    user_input_unique: UniqueView<UserInputUnique>,
    mut physics_context_unique: UniqueViewMut<PhysicsContextUnique>,
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

        let camera_rotation = rotation.rotation;

        let grab_config = GrabConfig {
            linear_acceleration: grab.params.linear_acceleration,
            linear_stiffness: grab.params.linear_stiffness,
            linear_damping: grab.params.linear_damping,

            angular_acceleration: grab.params.angular_acceleration,
            angular_stiffness: grab.params.angular_stiffness,
            angular_damping: grab.params.angular_damping,
        };

        if grab_just_pressed && grab.grab.is_none() {
            let Some(hit) = focus.hit else {
                continue;
            };

            let is_dynamic = hit.body.is_dynamic(&physics_context_unique.handle);

            if is_dynamic {
                grab.grab = physics_context_unique.handle.create_grab(hit.body, &grab_config, camera_rotation);
            }
        } else if !grab_pressed {
            if let Some(object_grab) = grab.grab.take() {
                object_grab.release(&mut physics_context_unique.handle);
            }
        }

        if let Some(object_grab) = &grab.grab {
            let position = origin + forward * grab.params.distance;

            object_grab.move_anchor(&mut physics_context_unique.handle, position, camera_rotation);
        }
    }
}
