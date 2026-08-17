use crate::chunk::region_coordinate::RegionCoordinate;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ChunkCoordinate {
    pub x: i32,
    pub z: i32,
}

impl ChunkCoordinate {
    pub const ORIGIN: Self = Self { x: 0, z: 0 };

    pub fn create(x: i32, z: i32) -> Self {
        Self { x, z }
    }

    pub fn offset(&self, x: i32, z: i32) -> Self {
        Self {
            x: self.x + x,
            z: self.z + z,
        }
    }

    pub fn region(&self) -> RegionCoordinate {
        RegionCoordinate::create(
            self.x >> RegionCoordinate::SHIFT,
            self.z >> RegionCoordinate::SHIFT,
        )
    }

    pub fn local_x(&self) -> u32 {
        (self.x & (RegionCoordinate::CHUNKS as i32 - 1)) as u32
    }

    pub fn local_z(&self) -> u32 {
        (self.z & (RegionCoordinate::CHUNKS as i32 - 1)) as u32
    }

    pub fn distance_to(&self, other: Self) -> f32 {
        let x = (self.x - other.x) as f32;
        let z = (self.z - other.z) as f32;

        (x * x + z * z).sqrt()
    }
}
