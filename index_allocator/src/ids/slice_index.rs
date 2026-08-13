#[derive(Copy, Clone)]
pub struct SliceIndex {
    pub value: u32
}

impl SliceIndex {
    pub const ZERO: SliceIndex = SliceIndex { value: 0 };

    pub fn from(value: u32) -> Self {
        Self { value }
    }
}
