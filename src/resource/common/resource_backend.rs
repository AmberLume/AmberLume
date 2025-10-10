use crate::resource::common::res_ref::{ResMeta, ResRef, ResState};
use anyhow::Result;

pub type ResourceKey = [u8; 16];

pub trait ResourceBackend: Send + Sync + 'static {
    type Config: Send + Sync + Clone + 'static;
    type DependenciesRefs: Send + Sync + 'static;
    type Dependencies: Send + Sync + 'static;
    type Output: Send + Sync + 'static;
    type Storage: Send + Sync + 'static;

    fn key_from(config: &Self::Config) -> ResourceKey;

    fn build_dependencies_refs(&self, config: &Self::Config) -> Self::DependenciesRefs;

    fn collect_dependencies(
        &self,
        dependencies_ref: &mut Self::DependenciesRefs,
    ) -> Self::Dependencies;

    fn create(
        &self,
        config: Self::Config,
        dependencies: Self::Dependencies,
    ) -> Result<Self::Output>;

    fn destroy(&self, output: Self::Output) -> Result<()>;

    fn get_storage(&self) -> Self::Storage;

    fn create_ref(config: &Self::Config) -> ResRef<Self::Config> {
        ResRef {
            meta: ResMeta {
                config: config.clone(),
            },
            state: ResState::New,
        }
    }
}
