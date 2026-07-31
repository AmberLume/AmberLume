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

#[derive(Copy, Clone)]
pub struct ChunkIndex {
    pub value: u32
}

impl ChunkIndex {
    pub const ZERO: ChunkIndex = ChunkIndex { value: 0 };
    
    pub fn from(value: u32) -> Self {
        Self { value }
    }
}
