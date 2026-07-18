use glam::Vec3;
use shipyard::{Get, IntoIter, UniqueView, UniqueViewMut, View, ViewMut};
use crate::input_handler::hardware_key_codes::HardwareKeyCode;
use crate::world::components::camera_component::CameraComponent;
use crate::world::components::position_component::PositionComponent;
use crate::world::components::rotation_component::RotationComponent;
use crate::world::physics::components::character_physics_component::CharacterPhysicsComponent;
use crate::world::unique::player_control_unique::PlayerControlUnique;
use crate::world::unique::user_input_unique::UserInputUnique;
use crate::world::unique::world_time_unique::WorldTimeUnique;

const FLY_SPEED: f32 = 8.0;

pub fn camera_fly_system(
    user_input_unique: UniqueView<UserInputUnique>,
    world_time_unique: UniqueView<WorldTimeUnique>,
    mut player_control_unique: UniqueViewMut<PlayerControlUnique>,
    character_physics: View<CharacterPhysicsComponent>,
    mut cameras: ViewMut<CameraComponent>,
    mut positions: ViewMut<PositionComponent>,
    rotations: View<RotationComponent>,
) {
    let input_frame = &user_input_unique.input_frame;
    let toggle = input_frame.just_pressed(HardwareKeyCode::F1);

    let return_target = (&character_physics).iter().with_id().next().map(|(id, _)| id);

    for (camera_id, camera) in (&mut cameras).iter().with_id() {
        if toggle {
            camera.target_id = match camera.target_id {
                Some(_) => None,
                None => return_target,
            };
        }

        player_control_unique.controlled = camera.target_id;

        if camera.target_id.is_some() {
            continue;
        }

        let Ok(rotation) = rotations.get(camera_id) else {
            continue;
        };

        let forward = rotation.rotation * Vec3::Z;
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

        if let Ok(mut position) = (&mut positions).get(camera_id) {
            position.position += direction.normalize_or_zero() * FLY_SPEED * speed_multiplier * world_time_unique.delta;
        }
    }
}
