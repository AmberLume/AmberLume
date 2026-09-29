#[derive(Clone, Copy)]
pub struct FrameSliceIndex {
    pub current: u32,
    pub previous: u32,
}

impl FrameSliceIndex {
    pub fn create(frame: u64, slice_count: u32) -> Self {
        let slice_count = slice_count as u64;

        Self {
            current: (frame % slice_count) as u32,
            previous: ((frame + slice_count - 1) % slice_count) as u32,
        }
    }
}
