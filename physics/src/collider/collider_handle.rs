use rapier3d::prelude::ColliderHandle as RapierColliderHandle;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ColliderHandle {
    pub(crate) inner: RapierColliderHandle,
}
