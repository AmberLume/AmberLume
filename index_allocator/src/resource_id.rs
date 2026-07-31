#[derive(Hash, PartialEq, Eq, Clone, Copy, Ord, PartialOrd)]
pub struct ResourceId {
    pub inner: u32,
}

impl ResourceId {
    pub fn from(inner: u32) -> Self {
        Self { inner }
    }
}
