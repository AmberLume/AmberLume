use crate::platform_providers::providers::Providers;
use anyhow::Result;

pub trait AmberLumeLifecycle {
    fn attach(&mut self, providers: Providers) -> Result<()>;

    fn detach(&mut self) -> Result<()>;

    fn is_attached(&self) -> bool;
}
