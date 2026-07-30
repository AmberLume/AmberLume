use glam::Vec3;

#[derive(Debug, Clone, Copy)]
pub struct CharacterMoveRequest {
    pub input_velocity: Vec3,
    pub velocity: Vec3,
    pub is_grounded: bool,
    pub push_force: f32,
}
