use glam::Vec2;
use render_snapshot::CameraView;
use shipyard::Unique;

#[derive(Unique, Debug, Clone, Copy, Default)]
pub struct RenderViewUnique {
    pub resolved_camera: CameraView,
    pub viewport: Vec2,
}

impl RenderViewUnique {
    pub fn new() -> Self {
        Self::default()
    }
}
