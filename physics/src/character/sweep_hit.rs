use glam::Vec3;
use crate::collider::ColliderHandle;

#[derive(Debug, Clone, Copy)]
pub struct SweepHit {
    pub collider: ColliderHandle,
    pub distance: f32,
    pub normal: Vec3,
}
