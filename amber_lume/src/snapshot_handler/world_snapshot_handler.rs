use crate::snapshot_handler::world_snapshot::WorldSnapshot;
use std::sync::{Arc, Mutex};

pub struct WorldSnapshotHandler {
    pub snapshot: Mutex<Arc<WorldSnapshot>>,
}

impl WorldSnapshotHandler {
    pub fn new() -> Self {
        let initial_snapshot = WorldSnapshot::default();

        Self {
            snapshot: Mutex::new(Arc::new(initial_snapshot)),
        }
    }

    pub fn push(&self, snapshot: WorldSnapshot) {
        let mut guard = self.snapshot.lock().unwrap();

        *guard = Arc::new(snapshot);
    }

    pub fn pull(&self) -> Arc<WorldSnapshot> {
        let guard = self.snapshot.lock().unwrap();

        guard.clone()
    }
}
