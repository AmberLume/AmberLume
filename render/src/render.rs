use crate::frame::render_frame::RenderFrame;
use crate::settings::render_settings::RenderSettings;
use crate::window::render_window::RenderWindow;
use crate::window::window_size::WindowSize;
use anyhow::Result;

pub trait Render {
    fn attach(&mut self, window: RenderWindow) -> Result<()>;

    fn detach(&mut self) -> Result<()>;

    fn resize(&mut self, size: WindowSize) -> Result<()>;

    fn update_settings(&mut self, settings: RenderSettings) -> Result<()>;

    fn render_frame(&mut self, frame: RenderFrame) -> Result<()>;

    fn destroy(self: Box<Self>) -> Result<()>;
}
