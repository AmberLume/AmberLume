use crate::chunk::{ChunkCoordinate, ChunkGeometry};
use crate::residency::residency_limits::ResidencyLimits;
use glam::Vec3;

pub struct ChunkSelection;

impl ChunkSelection {
    const SIDES: [(i32, i32); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];

    pub fn root_span(split_factor: f32) -> i32 {
        split_factor.ceil().max(1.0) as i32
    }

    pub fn anchor(previous: Vec3, observer: Vec3, limits: ResidencyLimits) -> Vec3 {
        let x = observer.x - previous.x;
        let z = observer.z - previous.z;

        if (x * x + z * z).sqrt() > limits.rebuild_margin {
            return observer;
        }

        previous
    }

    pub fn select(observer: Vec3, limits: ResidencyLimits) -> Vec<ChunkCoordinate> {
        let root = ChunkGeometry::chunk_of(observer, limits.max_level);
        let span = Self::root_span(limits.split_factor);

        let mut selected = Vec::new();

        for z in -span..=span {
            for x in -span..=span {
                Self::descend(root.offset(x, z), observer, limits, &mut selected);
            }
        }

        selected
    }

    pub fn level_deltas(
        coordinate: ChunkCoordinate,
        observer: Vec3,
        limits: ResidencyLimits,
    ) -> [u32; 4] {
        Self::SIDES.map(|(x, z)| {
            let probe = ChunkGeometry::chunk_center(coordinate.offset(x, z));

            Self::level_at(probe, observer, limits).saturating_sub(coordinate.level)
        })
    }

    fn level_at(point: Vec3, observer: Vec3, limits: ResidencyLimits) -> u32 {
        for level in (0..=limits.max_level).rev() {
            if !Self::splits(ChunkGeometry::chunk_of(point, level), observer, limits) {
                return level;
            }
        }

        0
    }

    fn descend(
        coordinate: ChunkCoordinate,
        observer: Vec3,
        limits: ResidencyLimits,
        selected: &mut Vec<ChunkCoordinate>,
    ) {
        if Self::splits(coordinate, observer, limits) {
            for child in coordinate.children() {
                Self::descend(child, observer, limits, selected);
            }

            return;
        }

        selected.push(coordinate);
    }

    fn splits(coordinate: ChunkCoordinate, observer: Vec3, limits: ResidencyLimits) -> bool {
        if coordinate.level == 0 {
            return false;
        }

        let reach = ChunkGeometry::chunk_size(coordinate.level) * limits.split_factor;

        ChunkGeometry::distance_to(observer, coordinate) < reach
    }
}
