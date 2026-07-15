use shipyard::{Component, EntityId};

#[derive(Component, Debug, Clone, Copy)]
pub struct CameraComponent {
    pub target_id: Option<EntityId>,

    pub fov: f32,
    pub near: f32,
    pub far: f32,
}
