use glam::Vec3;
use input::HardwareKeyCode;
use shipyard::{Get, IntoIter, UniqueView, UniqueViewMut, View, ViewMut};
use crate::world::components::camera_component::CameraComponent;
use crate::world::components::position_component::PositionComponent;
use crate::world::components::rotation_component::RotationComponent;
use crate::world::physics::components::character_physics_component::CharacterPhysicsComponent;
use crate::world::unique::player_control_unique::PlayerControlUnique;
use crate::world::unique::user_input_unique::UserInputUnique;
use crate::world::unique::world_time_unique::WorldTimeUnique;

const FLY_SPEED: f32 = 8.0;

pub fn camera_fly_system(
    mut user_input_unique: UniqueViewMut<UserInputUnique>,
    world_time_unique: UniqueView<WorldTimeUnique>,
    mut player_control_unique: UniqueViewMut<PlayerControlUnique>,
    character_physics: View<CharacterPhysicsComponent>,
    mut cameras: ViewMut<CameraComponent>,
    mut positions: ViewMut<PositionComponent>,
    rotations: View<RotationComponent>,
) {
    let Some(input) = user_input_unique.input.as_mut() else {
        return;
    };

    let toggle = input.key(HardwareKeyCode::F1, true).is_just_pressed();

    let move_forward = input.key(HardwareKeyCode::W, true).is_down();
    let move_back = input.key(HardwareKeyCode::S, true).is_down();
    let move_right = input.key(HardwareKeyCode::D, true).is_down();
    let move_left = input.key(HardwareKeyCode::A, true).is_down();
    let move_up = input.key(HardwareKeyCode::Space, true).is_down();
    let move_down = input.key(HardwareKeyCode::C, true).is_down();
    let move_fast = input.key(HardwareKeyCode::Shift, true).is_down();

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

        if move_forward {
            direction += forward;
        }

        if move_back {
            direction -= forward;
        }

        if move_right {
            direction += right;
        }

        if move_left {
            direction -= right;
        }

        if move_up {
            direction += Vec3::Y;
        }

        if move_down {
            direction -= Vec3::Y;
        }

        let speed_multiplier = if move_fast { 8.0 } else { 1.0 };

        if let Ok(mut position) = (&mut positions).get(camera_id) {
            position.position += direction.normalize_or_zero() * FLY_SPEED * speed_multiplier * world_time_unique.delta;
        }
    }
}
