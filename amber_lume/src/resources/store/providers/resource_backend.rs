use std::hash::{Hash, Hasher};
use ahash::AHasher;
use anyhow::Result;
use gpu::ManagedBufferFactory;
use crate::resources::store::providers::resource_provider::ResourceId;

#[derive(Hash, Eq, PartialEq)]
pub struct ResourceHash {
    pub value: u64,
}

pub trait ResourceBackend: Send + Sync + 'static {
    type Config: Send + Sync + Hash + Clone + 'static;
    type Output: Send + Sync + 'static;
    type Statistics;
    
    fn resource_hash_from(config: &Self::Config) -> ResourceHash {
        let mut hasher = AHasher::default();
        
        config.hash(&mut hasher);
        
        ResourceHash { value: hasher.finish() }
    }

    fn create(
        &self,
        id: &ResourceId,
        config: Self::Config,
    ) -> Result<Self::Output>;

    fn erase(&self, _id: &ResourceId) -> Result<()> { Ok(()) }

    fn statistics(&self) -> Self::Statistics;
    
    fn destroy_resource(&self, output: Self::Output) -> Result<()>;

    fn destroy(self, _buffer_factory: &ManagedBufferFactory) -> Result<()> where Self: Sized { Ok(()) }
}
