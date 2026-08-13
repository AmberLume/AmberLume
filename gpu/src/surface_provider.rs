use raw_window_handle::{RawDisplayHandle, RawWindowHandle};

pub trait SurfaceProvider: Send + Sync {
    fn handles(&self) -> (RawDisplayHandle, RawWindowHandle);

    fn size(&self) -> (u32, u32);
}
