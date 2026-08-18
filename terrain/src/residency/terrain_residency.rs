use crate::chunk::{ChunkCoordinate, ChunkGeometry};
use crate::residency::residency_update::ResidencyUpdate;
use glam::Vec3;
use std::collections::HashSet;

pub struct TerrainResidency {
    resident: HashSet<ChunkCoordinate>,
}

impl TerrainResidency {
    pub const DEFAULT_MAX_LEVEL: u32 = 5;
    pub const DEFAULT_SPLIT_FACTOR: f32 = 2.0;

    pub fn root_span(split_factor: f32) -> i32 {
        split_factor.ceil().max(1.0) as i32
    }

    pub fn create() -> Self {
        Self {
            resident: HashSet::new(),
        }
    }

    pub fn resident_count(&self) -> usize {
        self.resident.len()
    }

    pub fn is_resident(&self, coordinate: ChunkCoordinate) -> bool {
        self.resident.contains(&coordinate)
    }

    pub fn distance_to(observer: Vec3, coordinate: ChunkCoordinate) -> f32 {
        let center = ChunkGeometry::chunk_center(coordinate);
        let half_size = ChunkGeometry::half_size(coordinate.level);

        let x = ((observer.x - center.x).abs() - half_size).max(0.0);
        let z = ((observer.z - center.z).abs() - half_size).max(0.0);

        (x * x + z * z).sqrt()
    }

    pub fn update(&self, observer: Vec3, max_level: u32, split_factor: f32) -> ResidencyUpdate {
        let desired = Self::desired(observer, max_level, split_factor);

        ResidencyUpdate {
            requested: desired
                .iter()
                .filter(|coordinate| !self.resident.contains(coordinate))
                .copied()
                .collect(),
            retired: self
                .resident
                .iter()
                .filter(|coordinate| !desired.contains(coordinate))
                .copied()
                .collect(),
        }
    }

    pub fn mark_resident(&mut self, coordinate: ChunkCoordinate) {
        self.resident.insert(coordinate);
    }

    pub fn mark_released(&mut self, coordinate: ChunkCoordinate) {
        self.resident.remove(&coordinate);
    }

    pub fn desired(observer: Vec3, max_level: u32, split_factor: f32) -> HashSet<ChunkCoordinate> {
        let root = ChunkGeometry::chunk_of(observer, max_level);
        let span = Self::root_span(split_factor);

        let mut desired = HashSet::new();

        for z in -span..=span {
            for x in -span..=span {
                Self::subdivide(root.offset(x, z), observer, split_factor, &mut desired);
            }
        }

        desired
    }

    fn subdivide(
        coordinate: ChunkCoordinate,
        observer: Vec3,
        split_factor: f32,
        desired: &mut HashSet<ChunkCoordinate>,
    ) {
        let reach = ChunkGeometry::chunk_size(coordinate.level) * split_factor;

        if coordinate.level == 0 || Self::distance_to(observer, coordinate) >= reach {
            desired.insert(coordinate);

            return;
        }

        for child in coordinate.children() {
            Self::subdivide(child, observer, split_factor, desired);
        }
    }
}
