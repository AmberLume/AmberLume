#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct DataKey {
    pub handle: u32,
}

impl DataKey {
    pub fn new(handle: u32) -> Self {
        Self { handle }
    }
}
