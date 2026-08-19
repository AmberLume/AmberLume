use crate::chunk::{ChunkCoordinate, ChunkGeometry};
use crate::residency::residency_limits::ResidencyLimits;
use crate::residency::residency_update::ResidencyUpdate;
use glam::Vec3;
use std::collections::{HashMap, HashSet};

struct ResidencyWalk {
    visible: HashSet<ChunkCoordinate>,
    needed: HashSet<ChunkCoordinate>,
    missing: Vec<Vec<ChunkCoordinate>>,
}

pub struct TerrainResidency {
    resident: HashSet<ChunkCoordinate>,
    visible: HashSet<ChunkCoordinate>,
    idle: HashMap<ChunkCoordinate, u32>,
}

impl TerrainResidency {
    pub const SIDES: [(i32, i32); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];

    pub fn root_span(split_factor: f32) -> i32 {
        split_factor.ceil().max(1.0) as i32
    }

    pub fn create() -> Self {
        Self {
            resident: HashSet::new(),
            visible: HashSet::new(),
            idle: HashMap::new(),
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

    pub fn level_deltas(&self, coordinate: ChunkCoordinate, max_level: u32) -> [u32; 4] {
        Self::SIDES.map(|(x, z)| {
            self.neighbour_level(coordinate, x, z, max_level)
                .saturating_sub(coordinate.level)
        })
    }

    fn neighbour_level(&self, coordinate: ChunkCoordinate, x: i32, z: i32, max_level: u32) -> u32 {
        let probe = ChunkGeometry::chunk_center(coordinate.offset(x, z));

        for level in coordinate.level..=max_level {
            if self.visible.contains(&ChunkGeometry::chunk_of(probe, level)) {
                return level;
            }
        }

        coordinate.level
    }

    pub fn retired(&mut self, observer: Vec3, limits: ResidencyLimits) -> Vec<ChunkCoordinate> {
        let walk = self.walk(observer, limits);

        let pressure = self.resident.len() >= limits.capacity;

        let mut retired = Vec::new();

        for coordinate in self.resident.iter() {
            if walk.needed.contains(coordinate) {
                self.idle.remove(coordinate);

                continue;
            }

            let idle = self.idle.entry(*coordinate).or_default();

            *idle += 1;

            if pressure || *idle >= limits.retire_delay {
                retired.push(*coordinate);
            }
        }

        for coordinate in retired.iter() {
            self.idle.remove(coordinate);
        }

        retired
    }

    pub fn visible(&self, observer: Vec3, limits: ResidencyLimits) -> Vec<ChunkCoordinate> {
        self.walk(observer, limits).visible.into_iter().collect()
    }

    pub fn publish_visible(&mut self, visible: &[ChunkCoordinate]) {
        self.visible = visible.iter().copied().collect();
    }

    fn walk(&self, observer: Vec3, limits: ResidencyLimits) -> ResidencyWalk {
        let mut walk = ResidencyWalk {
            visible: HashSet::new(),
            needed: HashSet::new(),
            missing: Vec::new(),
        };

        let root = ChunkGeometry::chunk_of(observer, limits.max_level);
        let span = Self::root_span(limits.split_factor);

        for z in -span..=span {
            for x in -span..=span {
                self.traverse(root.offset(x, z), observer, limits, &mut walk);
            }
        }

        walk
    }

    pub fn update(&self, observer: Vec3, limits: ResidencyLimits) -> ResidencyUpdate {
        let walk = self.walk(observer, limits);

        let mut missing = walk.missing;

        let stale = self
            .resident
            .iter()
            .filter(|coordinate| !walk.needed.contains(coordinate))
            .count();

        let headroom = limits
            .capacity
            .saturating_sub(self.resident.len() - stale);

        missing.sort_by(|left, right| {
            right[0].level.cmp(&left[0].level).then_with(|| {
                Self::distance_to(observer, left[0])
                    .total_cmp(&Self::distance_to(observer, right[0]))
            })
        });

        let mut requested = Vec::new();

        for group in missing {
            if requested.len() + group.len() > limits.budget.min(headroom) {
                break;
            }

            requested.extend(group);
        }

        ResidencyUpdate {
            requested,
            visible: walk.visible.into_iter().collect(),
        }
    }

    pub fn mark_resident(&mut self, coordinate: ChunkCoordinate) {
        self.resident.insert(coordinate);
    }

    pub fn mark_released(&mut self, coordinate: ChunkCoordinate) {
        self.resident.remove(&coordinate);
        self.idle.remove(&coordinate);
    }

    fn traverse(
        &self,
        coordinate: ChunkCoordinate,
        observer: Vec3,
        limits: ResidencyLimits,
        walk: &mut ResidencyWalk,
    ) {
        walk.needed.insert(coordinate);

        let children = coordinate.children();
        let already_split = coordinate.level > 0 && children.iter().all(|child| self.covers(*child));

        if self.should_split(coordinate, observer, limits, already_split) {
            if already_split {
                for child in children {
                    self.traverse(child, observer, limits, walk);
                }

                return;
            }

            if self.resident.contains(&coordinate) {
                let pending = children
                    .iter()
                    .filter(|child| !self.covers(**child))
                    .copied()
                    .collect::<Vec<_>>();

                walk.missing.push(pending);
            } else {
                walk.missing.push(vec![coordinate]);
            }
        } else if !self.resident.contains(&coordinate) {
            walk.missing.push(vec![coordinate]);
        }

        walk.visible.insert(coordinate);
    }

    fn covers(&self, coordinate: ChunkCoordinate) -> bool {
        if self.resident.contains(&coordinate) {
            return true;
        }

        if coordinate.level == 0 {
            return false;
        }

        coordinate
            .children()
            .iter()
            .all(|child| self.covers(*child))
    }

    fn should_split(
        &self,
        coordinate: ChunkCoordinate,
        observer: Vec3,
        limits: ResidencyLimits,
        already_split: bool,
    ) -> bool {
        if coordinate.level == 0 {
            return false;
        }

        let reach = ChunkGeometry::chunk_size(coordinate.level) * limits.split_factor;

        let threshold = if already_split {
            reach * limits.hysteresis
        } else {
            reach
        };

        Self::distance_to(observer, coordinate) < threshold
    }
}
