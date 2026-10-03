use anyhow::Result;
use index_allocator::ResourceId;

pub trait ResourceBackend: Send + Sync + 'static {
    type Config: Send + Sync + Clone + 'static;
    type Output: Send + Sync + 'static;
    type Statistics;

    fn reserve(&self, _id: &ResourceId) -> Result<()> { Ok(()) }

    fn create(
        &self,
        id: &ResourceId,
        config: Self::Config,
    ) -> Result<Self::Output>;

    fn erase(&self, _id: &ResourceId) -> Result<()> { Ok(()) }

    fn statistics(&self) -> Self::Statistics;

    fn destroy_resource(&self, _output: Self::Output) -> Result<()> { Ok(()) }

    fn destroy(self) -> Result<()> where Self: Sized { Ok(()) }
}
