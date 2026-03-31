use parking_lot::Mutex;
use crate::resources::descriptor_index_managers::IndexManagers;
use crate::statistics::statistics::{Smooth, Statistics};

pub struct ResourceIndicesStatistics {
    internal: Mutex<ResourceIndicesStatisticsSnapshot>
}

impl Default for ResourceIndicesStatistics {
    fn default() -> Self {
        Self {
            internal: Mutex::new(ResourceIndicesStatisticsSnapshot {
                indices_used: None,
                vertices_used: None,
                
                skeleton_bones_used: None,

                texture_arrays_used: None,
                shadows_used: None,
                shadow_arrays_used: None,
            })
        }
    }
}

impl ResourceIndicesStatistics {
    pub fn fill(&self, index_managers: &IndexManagers) {
        let snapshot = ResourceIndicesStatisticsSnapshot {
            indices_used: Some(index_managers.index_index_manager.usage()),
            vertices_used: Some(index_managers.vertex_index_manager.usage()),

            skeleton_bones_used: Some(index_managers.skeleton_bones_index_manager.usage()),

            texture_arrays_used: Some(index_managers.texture_array_descriptors_index_manager.usage()),
            shadows_used: Some(index_managers.shadow_descriptors_index_manager.usage()),
            shadow_arrays_used: Some(index_managers.shadow_array_descriptors_index_manager.usage()),
        };

        let mut internal = self.internal.lock();
        *internal = snapshot;
    }
}

#[derive(Copy, Clone)]
pub struct ResourceIndicesStatisticsSnapshot {
    pub indices_used: Option<IndicesUsageStatistics>,
    pub vertices_used: Option<IndicesUsageStatistics>,
    
    pub skeleton_bones_used: Option<IndicesUsageStatistics>,
    
    pub texture_arrays_used: Option<IndicesUsageStatistics>,
    pub shadows_used: Option<IndicesUsageStatistics>,
    pub shadow_arrays_used: Option<IndicesUsageStatistics>,
}

#[derive(Copy, Clone)]
pub struct IndicesUsageStatistics {
    pub capacity: u32,

    pub used: u32,
    pub free: u32,

    pub grave: u32,
}

impl Smooth for ResourceIndicesStatisticsSnapshot {
    fn smooth(&self, other: &Self, _alpha: f32) -> Self {
        Self {
            ..*other
        }
    }
}

impl Statistics for ResourceIndicesStatistics {
    type Snapshot = ResourceIndicesStatisticsSnapshot;

    fn snapshot(&self) -> Self::Snapshot {
        let internal = *self.internal.lock();

        Self::Snapshot {
            ..internal
        }
    }
}
