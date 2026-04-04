#[derive(Clone, Copy)]
pub enum ImageSize {
    FullResolution,
    Absolute { width: u32, height: u32 },
}
