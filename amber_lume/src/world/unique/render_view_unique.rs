use crate::snapshot_handler::resolved_camera::ResolvedCamera;
use shipyard::Unique;

#[derive(Unique, Debug, Clone, Copy, Default)]
pub struct RenderViewUnique {
    pub resolved_camera: ResolvedCamera,
}

impl RenderViewUnique {
    pub fn new() -> Self {
        Self::default()
    }
}
