use crate::chunk::{ChunkCoordinate, ChunkGeometry};
use crate::residency::residency_update::ResidencyUpdate;
use glam::Vec3;
use std::collections::HashSet;

pub struct TerrainResidency {
    resident: HashSet<ChunkCoordinate>,
}

impl TerrainResidency {
    pub const DEFAULT_LOAD_DISTANCE: u32 = 4;

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

        let x = ((observer.x - center.x).abs() - ChunkGeometry::HALF_SIZE).max(0.0);
        let z = ((observer.z - center.z).abs() - ChunkGeometry::HALF_SIZE).max(0.0);

        (x * x + z * z).sqrt()
    }

    pub fn update(&self, observer: Vec3, load_distance: u32) -> ResidencyUpdate {
        let desired = self.desired(observer, load_distance);

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

    fn desired(&self, observer: Vec3, load_distance: u32) -> HashSet<ChunkCoordinate> {
        let center = ChunkGeometry::chunk_of(observer);
        let reach = load_distance as i32 + 1;
        let range = load_distance as f32 * ChunkGeometry::CHUNK_SIZE;

        let mut desired = HashSet::new();

        for z in -reach..=reach {
            for x in -reach..=reach {
                let coordinate = center.offset(x, z);

                if Self::distance_to(observer, coordinate) <= range {
                    desired.insert(coordinate);
                }
            }
        }

        desired
    }
}
