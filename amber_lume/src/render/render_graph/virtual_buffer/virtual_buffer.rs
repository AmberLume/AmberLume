#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct VirtualBuffer {
    pub handle: u32,
}

impl VirtualBuffer {
    pub fn new(handle: u32) -> Self {
        Self { handle }
    }
}
