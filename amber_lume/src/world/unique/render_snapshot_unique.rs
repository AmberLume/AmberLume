use crate::snapshot_handler::render_snapshot_handler::RenderSnapshotHandler;
use shipyard::Unique;
use std::sync::Arc;

#[derive(Unique)]
pub struct RenderSnapshotUnique {
    pub handler: Arc<RenderSnapshotHandler>,
}

impl RenderSnapshotUnique {
    pub fn new(render_snapshot_handler: Arc<RenderSnapshotHandler>) -> Self {
        Self {
            handler: render_snapshot_handler,
        }
    }
}
