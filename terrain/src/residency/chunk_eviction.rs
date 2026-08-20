use crate::chunk::{ChunkCoordinate, ChunkGeometry};
use glam::Vec3;
use std::collections::HashSet;

pub struct ChunkEviction;

impl ChunkEviction {
    pub fn excess(
        loaded: &[ChunkCoordinate],
        selected: &[ChunkCoordinate],
        observer: Vec3,
        capacity: usize,
    ) -> Vec<ChunkCoordinate> {
        let excess = loaded.len().saturating_sub(capacity);

        if excess == 0 {
            return Vec::new();
        }

        let selected = selected.iter().copied().collect::<HashSet<_>>();

        let mut evicted = loaded
            .iter()
            .filter(|coordinate| !selected.contains(coordinate))
            .copied()
            .collect::<Vec<_>>();

        evicted.sort_by(|left, right| {
            ChunkGeometry::distance_to(observer, *right)
                .total_cmp(&ChunkGeometry::distance_to(observer, *left))
        });

        evicted.truncate(excess);

        evicted
    }
}
