use anyhow::Result;
use std::sync::Arc;
use gpu::RenderTarget;

pub trait AmberLumeLifecycle {
    fn attach_render_target(&mut self, target: Arc<dyn RenderTarget>) -> Result<()>;

    fn detach_render_target(&mut self) -> Result<()>;

    fn is_render_target_attached(&self) -> bool;

    fn pause(&mut self);

    fn resume(&mut self);

    fn is_paused(&self) -> bool;
}
