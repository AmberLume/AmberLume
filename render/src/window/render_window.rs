use crate::window::window_size::WindowSize;
use raw_window_handle::{RawDisplayHandle, RawWindowHandle};

#[derive(Clone, Copy, Debug)]
pub struct RenderWindow {
    pub display: RawDisplayHandle,
    pub window: RawWindowHandle,

    pub size: WindowSize,
}
