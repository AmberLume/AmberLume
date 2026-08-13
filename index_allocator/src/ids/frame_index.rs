#[derive(Copy, Clone)]
pub struct FrameIndex {
    pub value: u32
}

impl FrameIndex {
    pub const ZERO: FrameIndex = FrameIndex { value: 0 };

    pub fn from(value: u32) -> Self {
        Self { value }
    }
}
