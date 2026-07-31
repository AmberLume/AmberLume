use std::hash::Hash;
use anyhow::Result;
use index_allocator::ResourceId;

pub trait ResourceBackend: Send + Sync + 'static {
    type Config: Send + Sync + Hash + Clone + 'static;
    type Output: Send + Sync + 'static;
    type Statistics;

    fn create(
        &self,
        id: &ResourceId,
        config: Self::Config,
    ) -> Result<Self::Output>;

    fn erase(&self, _id: &ResourceId) -> Result<()> { Ok(()) }

    fn statistics(&self) -> Self::Statistics;

    fn destroy_resource(&self, output: Self::Output) -> Result<()>;

    fn destroy(self) -> Result<()> where Self: Sized { Ok(()) }
}
