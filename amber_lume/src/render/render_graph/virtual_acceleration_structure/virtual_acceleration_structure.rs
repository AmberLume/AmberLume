#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct VirtualAccelerationStructure {
    pub handle: u32,
}

impl VirtualAccelerationStructure {
    pub fn new(handle: u32) -> Self {
        Self { handle }
    }
}
