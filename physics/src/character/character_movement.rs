use math::Vec3;

#[derive(Debug, Clone, Copy)]
pub struct CharacterMovement {
    pub translation: Vec3,
    pub velocity: Vec3,
    pub grounded: bool,
}
