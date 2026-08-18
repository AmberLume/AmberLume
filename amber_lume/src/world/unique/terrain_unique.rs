use glam::Vec3;
use index_allocator::Allocation;
use crate::render::frame_data::terrain_chunk_view::TerrainChunkView;
use crate::render::frame_data::terrain_generate_request::TerrainGenerateRequest;
use resource_residency::ResRef;
use resource_store::ResourceStore;
use shipyard::Unique;
use std::sync::Arc;
use terrain::{ProceduralTerrainSource, ResidencyLimits, TerrainResidency};

#[derive(Unique)]
pub struct TerrainUnique {
    pub material: Arc<ResRef>,

    pub source: ProceduralTerrainSource,
    pub residency: TerrainResidency,
    pub limits: ResidencyLimits,

    pub frozen_observer: Option<Vec3>,

    pub shared_indices: Option<Allocation>,

    pub generate_requests: Vec<TerrainGenerateRequest>,
    pub chunk_views: Vec<TerrainChunkView>,
}

impl TerrainUnique {
    pub fn new(resource_store: Arc<ResourceStore>) -> Self {
        Self {
            material: resource_store.persistent_resources.default_material(),

            source: ProceduralTerrainSource::create(),
            residency: TerrainResidency::create(),
            limits: ResidencyLimits::create(),

            frozen_observer: None,

            shared_indices: None,

            generate_requests: Vec::new(),
            chunk_views: Vec::new(),
        }
    }

    pub fn take_shared_indices(&mut self) -> Option<Allocation> {
        self.shared_indices.take()
    }
}
