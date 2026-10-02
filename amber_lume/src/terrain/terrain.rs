use crate::terrain::terrain_chunk_view::TerrainChunkView;
use crate::terrain::terrain_frame::TerrainFrame;
use crate::terrain::terrain_generate_request::TerrainGenerateRequest;
use crate::terrain::terrain_stitch_request::TerrainStitchRequest;
use crate::terrain::terrain_chunk::TerrainChunk;
use anyhow::Result;
use glam::{Mat4, Vec3};
use gpu_data::VertexNormalTangentGPU;
use gpu_data::VertexPositionGPU;
use gpu_data::VertexUvGPU;
use render_snapshot::{RenderEntity, RenderEntityId};
use resource_residency::ResRef;
use resource_residency::ResourceProvider;
use resource_store::{MeshBackend, MeshConfig, SubmeshConfig};
use std::collections::HashMap;
use std::mem::take;
use std::sync::Arc;
use terrain::{
    ChunkCoordinate, ChunkEviction, ChunkGeometry, ChunkPayload, ChunkSelection, ChunkTopology,
    ProceduralTerrainSource, ResidencyLimits, TerrainSource,
};
use tracing::error;

pub struct Terrain {
    mesh_provider: Arc<ResourceProvider<MeshBackend>>,

    topology: Arc<ResRef>,

    source: ProceduralTerrainSource,
    limits: ResidencyLimits,

    anchor: Option<Vec3>,

    chunks: HashMap<ChunkCoordinate, TerrainChunk>,

    generate_requests: Vec<TerrainGenerateRequest>,
    stitch_requests: Vec<TerrainStitchRequest>,
    chunk_views: Vec<TerrainChunkView>,
    drawables: Vec<RenderEntity>,
}

impl Terrain {
    pub fn new(
        mesh_provider: Arc<ResourceProvider<MeshBackend>>,
        material: Arc<ResRef>,
    ) -> Result<Self> {
        let nodes = (0..ChunkGeometry::NODE_COUNT)
            .map(|node| (node % ChunkGeometry::NODES, node / ChunkGeometry::NODES))
            .collect::<Vec<_>>();

        let positions = nodes
            .iter()
            .map(|(column, row)| {
                VertexPositionGPU::new(ChunkGeometry::node_local_position(0, *column as i32, *row as i32, 0.0).to_array())
            })
            .collect::<Vec<_>>();
        let normal_tangents = nodes
            .iter()
            .map(|_| VertexNormalTangentGPU::new([0.0, 1.0, 0.0], [1.0, 0.0, 0.0, 1.0]))
            .collect::<Vec<_>>();
        let uvs = nodes
            .iter()
            .map(|(column, row)| {
                VertexUvGPU::new([
                    *column as f32 / ChunkGeometry::CELLS as f32,
                    *row as f32 / ChunkGeometry::CELLS as f32,
                ])
            })
            .collect::<Vec<_>>();

        let half_size = ChunkGeometry::half_size(0);

        let topology = mesh_provider.reserve()?;

        mesh_provider.write(&topology, MeshConfig::Data {
            submeshes: vec![SubmeshConfig {
                indices: ChunkTopology::build().indices().to_vec(),

                positions,
                normal_tangents,
                uvs,
                skins: Vec::new(),

                material,
                bounds: [-half_size, 0.0, -half_size, half_size, 0.0, half_size],
            }],
            bindings: Vec::new(),

            skeleton: None,
        })?;

        Ok(Self {
            mesh_provider,

            topology,

            source: ProceduralTerrainSource::create(),
            limits: ResidencyLimits::create(),

            anchor: None,

            chunks: HashMap::new(),

            generate_requests: Vec::new(),
            stitch_requests: Vec::new(),
            chunk_views: Vec::new(),
            drawables: Vec::new(),
        })
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

    pub fn chunks_for(&mut self, observer: Vec3) -> &[RenderEntity] {
        let observer = self.anchored(observer);

        let selected = ChunkSelection::select(observer, self.limits);

        let topology_resident = self.mesh_provider.backend
            .with_mesh(self.topology.id, |_| ())
            .is_some();

        if topology_resident {
            for coordinate in &selected {
                self.load(*coordinate);
            }
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

    fn load(&mut self, coordinate: ChunkCoordinate) {
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

        let chunk = match self.allocate(payload) {
            Ok(chunk) => chunk,
            Err(error) => {
                error!("Failed to reserve terrain mesh {coordinate:?}: {:#}", error);

                return;
            }
        };

        self.generate_requests.push(TerrainGenerateRequest {
            mesh_id: chunk.mesh.id,

            cell_size: ChunkGeometry::cell_size(coordinate.level),

            heights: chunk.payload.heights().to_vec(),
        });

        self.chunks.insert(coordinate, chunk);
    }

    fn allocate(&self, payload: ChunkPayload) -> Result<TerrainChunk> {
        let mesh = self.mesh_provider.reserve()?;

        self.mesh_provider.write(&mesh, MeshConfig::Instance {
            source: self.topology.clone(),

            vertex_slice_count: 1,
            bounds: Some(payload.bounds()),
        })?;

        Ok(TerrainChunk {
            payload: Box::new(payload),

            mesh,

            level_deltas: [u32::MAX; 4],
        })
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

                mesh_id: chunk.mesh.id.inner,
                animation: None,
                outline: [0.0; 4],
            });

            chunk_views.push(TerrainChunkView {
                center,
                level: coordinate.level,

                mesh_id: chunk.mesh.id,
            });

            let level_deltas = ChunkSelection::level_deltas(*coordinate, observer, limits);

            if level_deltas == chunk.level_deltas {
                continue;
            }

            chunk.level_deltas = level_deltas;

            stitch_requests.push(TerrainStitchRequest {
                mesh_id: chunk.mesh.id,
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
