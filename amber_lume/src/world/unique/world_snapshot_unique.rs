use crate::snapshot_handler::world_snapshot_handler::WorldSnapshotHandler;
use shipyard::Unique;
use std::sync::Arc;

#[derive(Unique)]
pub struct WorldSnapshotUnique {
    pub handler: Arc<WorldSnapshotHandler>,
}

impl WorldSnapshotUnique {
    pub fn new(world_snapshot_handler: Arc<WorldSnapshotHandler>) -> Self {
        Self {
            handler: world_snapshot_handler,
        }
    }
}
