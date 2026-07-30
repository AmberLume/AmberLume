use crate::snapshot_handler::camera_view::CameraView;
use shipyard::Unique;

#[derive(Unique, Debug, Clone, Copy, Default)]
pub struct RenderViewUnique {
    pub resolved_camera: CameraView,
}

impl RenderViewUnique {
    pub fn new() -> Self {
        Self::default()
    }
}
