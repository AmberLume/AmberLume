use crate::settings::explicit_rhi_settings::ExplicitRhiSettings;
use crate::window::present_settings::PresentSettings;
use crate::window::window::Window;
use crate::window::window_size::WindowSize;
use anyhow::Result;

pub trait ExplicitRhi: Sized {
    fn create(settings: ExplicitRhiSettings) -> Result<Self>;

    fn attach(&mut self, window: Window) -> Result<()>;

    fn detach(&mut self) -> Result<()>;

    fn resize(&mut self, size: WindowSize) -> Result<()>;

    fn set_present_settings(&mut self, settings: PresentSettings) -> Result<()>;

    fn destroy(self) -> Result<()>;
}
