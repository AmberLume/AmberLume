use crate::render::frame_data::terrain_chunk_view::TerrainChunkView;
use crate::render::frame_data::terrain_frame::TerrainFrame;
use crate::render::frame_data::terrain_generate_request::TerrainGenerateRequest;
use crate::render::frame_data::terrain_stitch_request::TerrainStitchRequest;
use crate::terrain::terrain_chunk::TerrainChunk;
use glam::{Mat4, Vec3};
use render_snapshot::{RenderEntity, RenderEntityId};
use resource_residency::{ResRef, ResourceProvider};
use resource_store::{MeshBackend, MeshConfig, ResourceStore};
use std::collections::HashMap;
use std::mem::take;
use std::sync::Arc;
use terrain::{
    ChunkCoordinate, ChunkEviction, ChunkGeometry, ChunkSelection, ChunkTopology,
    ProceduralTerrainSource, ResidencyLimits, TerrainSource,
};
use tracing::error;

pub struct Terrain {
    material: Arc<ResRef>,

    source: ProceduralTerrainSource,
    limits: ResidencyLimits,

    anchor: Option<Vec3>,

    chunks: HashMap<ChunkCoordinate, TerrainChunk>,

    topology: Arc<[u32]>,

    generate_requests: Vec<TerrainGenerateRequest>,
    stitch_requests: Vec<TerrainStitchRequest>,
    chunk_views: Vec<TerrainChunkView>,
    drawables: Vec<RenderEntity>,
}

impl Terrain {
    pub fn new(resource_store: Arc<ResourceStore>) -> Self {
        Self {
            material: resource_store.persistent_resources.default_material(),

            source: ProceduralTerrainSource::create(),
            limits: ResidencyLimits::create(),

            anchor: None,

            chunks: HashMap::new(),

            topology: Arc::from(ChunkTopology::build().indices()),

            generate_requests: Vec::new(),
            stitch_requests: Vec::new(),
            chunk_views: Vec::new(),
            drawables: Vec::new(),
        }
    }

    pub fn chunk(&self, coordinate: ChunkCoordinate) -> Option<&TerrainChunk> {
        self.chunks.get(&coordinate)
    }

    pub fn append_drawables(&mut self, entities: &mut Vec<RenderEntity>) {
        entities.append(&mut self.drawables);
    }

    pub fn take_frame(&mut self) -> TerrainFrame {
        TerrainFrame {
            generate_requests: take(&mut self.generate_requests),
            stitch_requests: take(&mut self.stitch_requests),

            chunks: take(&mut self.chunk_views),
        }
    }

    pub fn chunks_for(
        &mut self,
        observer: Vec3,
        mesh_provider: &ResourceProvider<MeshBackend>,
    ) -> &[RenderEntity] {
        let observer = self.anchored(observer);

        let selected = ChunkSelection::select(observer, self.limits);

        for coordinate in &selected {
            self.load(*coordinate, mesh_provider);
        }

        self.publish(&selected, observer);
        self.evict(&selected, observer);

        &self.drawables
    }

    fn anchored(&mut self, observer: Vec3) -> Vec3 {
        let anchor = match self.anchor {
            Some(anchor) => ChunkSelection::anchor(anchor, observer, self.limits),
            None => observer,
        };

        self.anchor = Some(anchor);

        anchor
    }

    fn load(&mut self, coordinate: ChunkCoordinate, mesh_provider: &ResourceProvider<MeshBackend>) {
        if self.chunks.contains_key(&coordinate) {
            return;
        }

        let payload = match self.source.load(coordinate) {
            Ok(payload) => payload,
            Err(error) => {
                error!("Failed to load terrain chunk {coordinate:?}: {:#}", error);

                return;
            }
        };

        let mesh_config = MeshConfig::Reserved {
            key: TerrainChunk::key(coordinate),
            indices: self.topology.clone(),
            vertex_count: ChunkGeometry::NODE_COUNT,
            material: self.material.clone(),
            bounds: payload.bounds(),
        };

        let handle = match mesh_provider.acquire_sync(mesh_config) {
            Ok(handle) => handle,
            Err(error) => {
                error!("Failed to reserve terrain mesh {coordinate:?}: {:#}", error);

                return;
            }
        };

        self.generate_requests.push(TerrainGenerateRequest {
            mesh_id: handle.id,

            cell_size: ChunkGeometry::cell_size(coordinate.level),

            heights: payload.heights().to_vec(),
        });

        self.chunks.insert(
            coordinate,
            TerrainChunk {
                payload: Box::new(payload),
                handle,
                level_deltas: [u32::MAX; 4],
            },
        );
    }

    fn publish(&mut self, selected: &[ChunkCoordinate], observer: Vec3) {
        let limits = self.limits;

        let Self {
            chunks,
            stitch_requests,
            chunk_views,
            drawables,
            ..
        } = self;

        drawables.clear();
        chunk_views.clear();

        for coordinate in selected {
            let Some(chunk) = chunks.get_mut(coordinate) else {
                continue;
            };

            let center = ChunkGeometry::chunk_center(*coordinate);

            drawables.push(RenderEntity {
                id: RenderEntityId::STATIC,

                transform_matrix: Mat4::from_translation(center),

                mesh_id: chunk.handle.id.inner,
                animation: None,
                outline: [0.0; 4],
            });

            chunk_views.push(TerrainChunkView {
                center,
                level: coordinate.level,

                mesh_id: chunk.handle.id,
            });

            let level_deltas = ChunkSelection::level_deltas(*coordinate, observer, limits);

            if level_deltas == chunk.level_deltas {
                continue;
            }

            chunk.level_deltas = level_deltas;

            stitch_requests.push(TerrainStitchRequest {
                mesh_id: chunk.handle.id,
                level_deltas: Self::pack_level_deltas(level_deltas),

                edge_heights: chunk.payload.edge_heights(),
            });
        }
    }

    fn evict(&mut self, selected: &[ChunkCoordinate], observer: Vec3) {
        if self.chunks.len() <= self.limits.capacity {
            return;
        }

        let loaded = self.chunks.keys().copied().collect::<Vec<_>>();

        for coordinate in
            ChunkEviction::excess(&loaded, selected, observer, self.limits.capacity)
        {
            self.chunks.remove(&coordinate);
        }
    }

    fn pack_level_deltas(level_deltas: [u32; 4]) -> u32 {
        level_deltas
            .iter()
            .enumerate()
            .fold(0, |packed, (side, delta)| {
                packed | ((delta & 0xFF) << (side * 8))
            })
    }
}
