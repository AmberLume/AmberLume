use crate::render_snapshot::RenderSnapshot;
use arc_swap::ArcSwapOption;
use std::sync::Arc;

pub struct RenderSnapshotHandler {
    pub snapshot: ArcSwapOption<RenderSnapshot>,
}

impl RenderSnapshotHandler {
    pub fn new() -> Self {
        Self {
            snapshot: ArcSwapOption::new(None),
        }
    }

    pub fn push(&self, snapshot: RenderSnapshot) {
        self.snapshot.store(Some(Arc::new(snapshot)));
    }

    pub fn pull(&self) -> Option<Arc<RenderSnapshot>> {
        self.snapshot.load().clone()
    }
}
