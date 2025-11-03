use crate::data::physical_size::PhysicalSize;
use raw_window_handle::{RawDisplayHandle, RawWindowHandle};

pub trait SurfaceProvider: Send + Sync {
    fn handles(&self) -> (RawDisplayHandle, RawWindowHandle);

    fn size(&self) -> PhysicalSize;
}
