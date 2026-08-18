use crate::common::fake_backend::FakeBackend;
use crate::common::manual_task_scheduler::ManualTaskScheduler;
use resource_residency::ResourceProvider;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};

pub struct Harness {
    pub provider: Arc<ResourceProvider<FakeBackend>>,

    scheduler: Arc<ManualTaskScheduler>,

    created: Arc<AtomicU32>,
    destroyed: Arc<AtomicU32>,
    erased: Arc<AtomicU32>,

    frame_counter: Arc<AtomicU64>,
}

impl Harness {
    pub const FRAMES_IN_FLIGHT: u32 = 2;
    pub const CAPACITY: u32 = 4;
    pub const MAX_ADVANCE_FRAMES: u32 = 64;

    pub fn create() -> Self {
        Self::with_failures(0)
    }

    pub fn with_failures(failures: u32) -> Self {
        let created = Arc::new(AtomicU32::new(0));
        let destroyed = Arc::new(AtomicU32::new(0));
        let erased = Arc::new(AtomicU32::new(0));
        let frame_counter = Arc::new(AtomicU64::new(0));
        let scheduler = Arc::new(ManualTaskScheduler::create());

        let backend = FakeBackend::create(
            created.clone(),
            destroyed.clone(),
            erased.clone(),
            Arc::new(AtomicU32::new(failures)),
        );

        Self {
            provider: ResourceProvider::with_scheduler(
                backend,
                Self::CAPACITY,
                Self::FRAMES_IN_FLIGHT,
                frame_counter.clone(),
                scheduler.clone(),
            ),

            scheduler,

            created,
            destroyed,
            erased,

            frame_counter,
        }
    }

    pub fn advance(&self, frames: u32) {
        for _ in 0..frames {
            self.provider.update();
            self.frame_counter.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn advance_until(&self, label: &str, condition: impl Fn() -> bool) {
        for _ in 0..Self::MAX_ADVANCE_FRAMES {
            if condition() {
                return;
            }

            self.provider.update();
            self.frame_counter.fetch_add(1, Ordering::Relaxed);
        }

        panic!(
            "condition '{}' was not reached in {} frames",
            label,
            Self::MAX_ADVANCE_FRAMES
        );
    }

    pub fn run_scheduled_creations(&self) -> u32 {
        self.scheduler.run_pending()
    }

    pub fn scheduled_creations(&self) -> usize {
        self.scheduler.pending()
    }

    pub fn created(&self) -> u32 {
        self.created.load(Ordering::Relaxed)
    }

    pub fn destroyed(&self) -> u32 {
        self.destroyed.load(Ordering::Relaxed)
    }

    pub fn erased(&self) -> u32 {
        self.erased.load(Ordering::Relaxed)
    }
}
