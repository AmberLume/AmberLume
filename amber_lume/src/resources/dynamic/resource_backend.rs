use anyhow::Result;
use crate::resources::dynamic::resource_provider::ResourceId;

pub type ResourceKey = [u8; 16];

pub trait ResourceBackend: Send + Sync + 'static {
    type Config: Send + Sync + Clone + 'static;
    type Output: Send + Sync + 'static;

    fn key_from(config: &Self::Config) -> ResourceKey;

    fn create(
        &self,
        id: &ResourceId,
        config: Self::Config,
    ) -> Result<Self::Output>;

    fn erase(&self, _id: &ResourceId) -> Result<()> { Ok(()) }

    fn destroy_resource(&self, output: Self::Output) -> Result<()>;
}
