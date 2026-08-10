#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct VirtualImage {
    pub handle: u32,
}

impl VirtualImage {
    pub fn new(handle: u32) -> Self {
        Self { handle }
    }
}
