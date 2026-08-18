use crate::chunk::region_coordinate::RegionCoordinate;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ChunkCoordinate {
    pub x: i32,
    pub z: i32,
    pub level: u32,
}

impl ChunkCoordinate {
    pub const ORIGIN: Self = Self { x: 0, z: 0, level: 0 };

    pub fn create(x: i32, z: i32, level: u32) -> Self {
        Self { x, z, level }
    }

    pub fn offset(&self, x: i32, z: i32) -> Self {
        Self {
            x: self.x + x,
            z: self.z + z,

            level: self.level,
        }
    }

    pub fn parent(&self) -> Self {
        Self {
            x: self.x >> 1,
            z: self.z >> 1,

            level: self.level + 1,
        }
    }

    pub fn children(&self) -> [Self; 4] {
        let level = self.level.saturating_sub(1);
        let x = self.x * 2;
        let z = self.z * 2;

        [
            Self { x, z, level },
            Self { x: x + 1, z, level },
            Self { x, z: z + 1, level },
            Self { x: x + 1, z: z + 1, level },
        ]
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
}
