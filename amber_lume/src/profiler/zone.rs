#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ZoneId(u32);

impl ZoneId {
    pub fn new(index: u32) -> Self {
        Self(index)
    }

    pub fn index(self) -> usize {
        self.0 as usize
    }

    pub fn value(self) -> u32 {
        self.0
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ZoneKind {
    Cpu,
    Gpu,
}
