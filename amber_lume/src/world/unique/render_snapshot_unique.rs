use render_snapshot::RenderSnapshot;
use shipyard::Unique;

#[derive(Unique)]
pub struct RenderSnapshotUnique {
    pub snapshot: Option<RenderSnapshot>,
}

impl RenderSnapshotUnique {
    pub fn new() -> Self {
        Self {
            snapshot: None,
        }
    }
}
