use crate::common::fake_resource::FakeResource;
use anyhow::{Result, bail};
use index_allocator::ResourceId;
use resource_residency::ResourceBackend;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

pub struct FakeBackend {
    created: Arc<AtomicU32>,
    destroyed: Arc<AtomicU32>,
    erased: Arc<AtomicU32>,
    failures_left: Arc<AtomicU32>,
}

impl FakeBackend {
    pub fn create(
        created: Arc<AtomicU32>,
        destroyed: Arc<AtomicU32>,
        erased: Arc<AtomicU32>,
        failures_left: Arc<AtomicU32>,
    ) -> Self {
        Self {
            created,
            destroyed,
            erased,
            failures_left,
        }
    }
}

impl ResourceBackend for FakeBackend {
    type Config = u32;
    type Output = FakeResource;
    type Statistics = ();

    fn create(&self, _id: &ResourceId, config: Self::Config) -> Result<Self::Output> {
        let failed = self
            .failures_left
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |left| {
                left.checked_sub(1)
            })
            .is_ok();

        if failed {
            bail!("fake backend failure");
        }

        self.created.fetch_add(1, Ordering::Relaxed);

        Ok(FakeResource { config })
    }

    fn erase(&self, _id: &ResourceId) -> Result<()> {
        self.erased.fetch_add(1, Ordering::Relaxed);

        Ok(())
    }

    fn statistics(&self) -> Self::Statistics {}

    fn destroy_resource(&self, _output: Self::Output) -> Result<()> {
        self.destroyed.fetch_add(1, Ordering::Relaxed);

        Ok(())
    }
}
