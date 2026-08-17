#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct RegionCoordinate {
    pub x: i32,
    pub z: i32,
}

impl RegionCoordinate {
    pub const CHUNKS: u32 = 64;
    pub const SHIFT: u32 = Self::CHUNKS.trailing_zeros();

    pub fn create(x: i32, z: i32) -> Self {
        Self { x, z }
    }
}
