use index_allocator::Allocation;
use crate::render::frame_data::terrain_generate_request::TerrainGenerateRequest;
use resource_residency::ResRef;
use resource_store::ResourceStore;
use shipyard::Unique;
use std::sync::Arc;
use terrain::{ProceduralTerrainSource, TerrainResidency};

#[derive(Unique)]
pub struct TerrainUnique {
    pub material: Arc<ResRef>,

    pub source: ProceduralTerrainSource,
    pub residency: TerrainResidency,
    pub max_level: u32,
    pub split_factor: f32,

    pub shared_indices: Option<Allocation>,

    pub generate_requests: Vec<TerrainGenerateRequest>,
}

impl TerrainUnique {
    pub fn new(resource_store: Arc<ResourceStore>) -> Self {
        Self {
            material: resource_store.persistent_resources.default_material(),

            source: ProceduralTerrainSource::create(),
            residency: TerrainResidency::create(),
            max_level: TerrainResidency::DEFAULT_MAX_LEVEL,
            split_factor: TerrainResidency::DEFAULT_SPLIT_FACTOR,

            shared_indices: None,

            generate_requests: Vec::new(),
        }
    }

    pub fn take_shared_indices(&mut self) -> Option<Allocation> {
        self.shared_indices.take()
    }
}
