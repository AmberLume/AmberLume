use crate::resources::common::res_ref::{ResMeta, ResRef, ResState};
use anyhow::Result;
use std::fmt::Debug;

pub type ResourceKey = [u8; 16];

pub trait ResourceBackend: Send + Sync + 'static {
    type Config: Send + Sync + Clone + Debug + 'static;
    type Dependencies: Send + Sync + 'static;
    type Output: Send + Sync + Clone + 'static;

    fn key_from(config: &Self::Config) -> ResourceKey;

    fn collect_dependencies(&self, config: &Self::Config) -> Self::Dependencies;

    fn create(
        &self,
        config: Self::Config,
        dependencies: Self::Dependencies,
    ) -> Result<Self::Output>;

    fn destroy(&self, output: Self::Output) -> Result<()>;

    fn create_ref(config: &Self::Config) -> ResRef<Self::Config> {
        ResRef {
            meta: ResMeta {
                config: config.clone(),
            },
            state: ResState::New,
        }
    }
}
