#[derive(Debug, Copy, Clone)]
pub enum MetaValue {
    U32(u32),
    U64(u64),
    F32(f32),
    F64(f64),
}

impl From<u32> for MetaValue {
    fn from(value: u32) -> Self {
        Self::U32(value)
    }
}

impl From<u64> for MetaValue {
    fn from(value: u64) -> Self {
        Self::U64(value)
    }
}

impl From<f32> for MetaValue {
    fn from(value: f32) -> Self {
        Self::F32(value)
    }
}

impl From<f64> for MetaValue {
    fn from(value: f64) -> Self {
        Self::F64(value)
    }
}
