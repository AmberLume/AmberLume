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

                meshes_used: None,
                submeshes_used: None,
                materials_used: None,

                textures_used: None,
                texture_arrays_used: None,
                shadows_used: None,
                shadow_arrays_used: None,

                pipelines_used: None,
                compute_pipelines_used: None,
            })
        }
    }
}

impl ResourceIndicesStatistics {
    pub fn fill(&self, index_managers: &IndexManagers) {
        let snapshot = ResourceIndicesStatisticsSnapshot {
            indices_used: Some(index_managers.index_index_manager.usage()),
            vertices_used: Some(index_managers.vertex_index_manager.usage()),

            meshes_used: Some(index_managers.mesh_index_manager.usage()),
            submeshes_used: Some(index_managers.submesh_index_manager.usage()),
            materials_used: Some(index_managers.material_index_manager.usage()),

            textures_used: Some(index_managers.texture_descriptors_index_manager.usage()),
            texture_arrays_used: Some(index_managers.texture_array_descriptors_index_manager.usage()),
            shadows_used: Some(index_managers.shadow_descriptors_index_manager.usage()),
            shadow_arrays_used: Some(index_managers.shadow_array_descriptors_index_manager.usage()),

            pipelines_used: Some(index_managers.pipeline_index_manager.usage()),
            compute_pipelines_used: Some(index_managers.compute_pipeline_index_manager.usage()),
        };

        let mut internal = self.internal.lock();
        *internal = snapshot;
    }
}

#[derive(Copy, Clone)]
pub struct ResourceIndicesStatisticsSnapshot {
    pub indices_used: Option<IndicesUsageStatistics>,
    pub vertices_used: Option<IndicesUsageStatistics>,

    pub meshes_used: Option<IndicesUsageStatistics>,
    pub submeshes_used: Option<IndicesUsageStatistics>,
    pub materials_used: Option<IndicesUsageStatistics>,

    pub textures_used: Option<IndicesUsageStatistics>,
    pub texture_arrays_used: Option<IndicesUsageStatistics>,
    pub shadows_used: Option<IndicesUsageStatistics>,
    pub shadow_arrays_used: Option<IndicesUsageStatistics>,

    pub pipelines_used: Option<IndicesUsageStatistics>,
    pub compute_pipelines_used: Option<IndicesUsageStatistics>,
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
