use crate::terrain::terrain_chunk_view::TerrainChunkView;
use crate::terrain::terrain_frame::TerrainFrame;
use crate::terrain::terrain_generate_request::TerrainGenerateRequest;
use crate::terrain::terrain_stitch_request::TerrainStitchRequest;
use crate::terrain::terrain_chunk::TerrainChunk;
use anyhow::{bail, Context, Result};
use glam::{Mat4, Vec3};
use gpu::RangeAllocation;
use gpu::ResourceTransfer;
use gpu_data::MeshVertexAttributeGPU;
use gpu_data::MeshVertexGPU;
use gpu_data::SubmeshGPU;
use index_allocator::Allocation;
use index_allocator::DeferredDestroy;
use render_snapshot::{RenderEntity, RenderEntityId};
use resource_residency::ResRef;
use resource_store::{GeometryRange, MeshTable};
use std::collections::HashMap;
use std::mem::take;
use std::sync::Arc;
use terrain::{
    ChunkCoordinate, ChunkEviction, ChunkGeometry, ChunkPayload, ChunkSelection, ChunkTopology,
    ProceduralTerrainSource, ResidencyLimits, TerrainSource,
};
use tracing::error;

pub struct Terrain {
    material: Arc<ResRef>,

    mesh_table: Arc<MeshTable>,

    index: Arc<RangeAllocation<u32>>,
    mesh_vertex: Arc<RangeAllocation<MeshVertexGPU>>,
    mesh_vertex_attribute: Arc<RangeAllocation<MeshVertexAttributeGPU>>,

    deferred_destroy: Arc<DeferredDestroy>,

    source: ProceduralTerrainSource,
    limits: ResidencyLimits,

    anchor: Option<Vec3>,

    chunks: HashMap<ChunkCoordinate, TerrainChunk>,

    topology: Allocation,

    generate_requests: Vec<TerrainGenerateRequest>,
    stitch_requests: Vec<TerrainStitchRequest>,
    chunk_views: Vec<TerrainChunkView>,
    drawables: Vec<RenderEntity>,
}

impl Terrain {
    pub fn new(
        mesh_table: Arc<MeshTable>,
        index: Arc<RangeAllocation<u32>>,
        mesh_vertex: Arc<RangeAllocation<MeshVertexGPU>>,
        mesh_vertex_attribute: Arc<RangeAllocation<MeshVertexAttributeGPU>>,
        material: Arc<ResRef>,
        resource_transfer: Arc<ResourceTransfer>,
        deferred_destroy: Arc<DeferredDestroy>,
    ) -> Result<Self> {
        let topology = ChunkTopology::build();

        let topology_allocation = index.allocator.allocate(topology.index_count())
            .with_context(|| format!("Failed to allocate {} terrain indices", topology.index_count()))?;

        resource_transfer.load_buffer_at(
            index.slice(topology_allocation.offset, topology_allocation.size),
            topology.indices(),
        )?;

        Ok(Self {
            material,

            mesh_table,

            index,
            mesh_vertex,
            mesh_vertex_attribute,

            deferred_destroy,

            source: ProceduralTerrainSource::create(),
            limits: ResidencyLimits::create(),

            anchor: None,

            chunks: HashMap::new(),

            topology: topology_allocation,

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

        for coordinate in &selected {
            self.load(*coordinate);
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
            mesh_id: chunk.mesh_id,

            cell_size: ChunkGeometry::cell_size(coordinate.level),

            heights: chunk.payload.heights().to_vec(),
        });

        self.chunks.insert(coordinate, chunk);
    }

    fn allocate(&self, payload: ChunkPayload) -> Result<TerrainChunk> {
        let mesh_id = self.mesh_table.mesh.allocator.acquire();
        let vertices_allocation = self.mesh_vertex.allocator.allocate(ChunkGeometry::NODE_COUNT);
        let vertex_attributes_allocation = self.mesh_vertex_attribute.allocator.allocate(ChunkGeometry::NODE_COUNT);
        let submeshes_allocation = self.mesh_table.submesh.allocator.allocate(1);

        let (Some(mesh_id), Some(vertices_allocation), Some(vertex_attributes_allocation), Some(submeshes_allocation)) =
            (mesh_id, vertices_allocation, vertex_attributes_allocation, submeshes_allocation)
        else {
            if let Some(mesh_id) = mesh_id {
                self.mesh_table.mesh.allocator.release(mesh_id);
            }
            if let Some(vertices_allocation) = vertices_allocation {
                self.mesh_vertex.allocator.release(vertices_allocation);
            }
            if let Some(vertex_attributes_allocation) = vertex_attributes_allocation {
                self.mesh_vertex_attribute.allocator.release(vertex_attributes_allocation);
            }
            if let Some(submeshes_allocation) = submeshes_allocation {
                self.mesh_table.submesh.allocator.release(submeshes_allocation);
            }

            bail!("Resource buffers are full");
        };

        let submesh = SubmeshGPU::create(
            self.topology.size,
            self.topology.offset,
            vertices_allocation.offset,
            vertex_attributes_allocation.offset,
            self.material.id.inner,
            payload.bounds(),
        );

        self.mesh_table.write(
            mesh_id,
            submeshes_allocation,
            &[submesh],
            0,
            vec![GeometryRange {
                index_count: self.topology.size,
                index_offset: self.topology.offset,
                vertex_offset: vertices_allocation.offset,
                vertex_count: vertices_allocation.size,
            }],
        )?;

        Ok(TerrainChunk {
            payload: Box::new(payload),

            mesh_id,
            vertices_allocation,
            vertex_attributes_allocation,
            submeshes_allocation,

            level_deltas: [u32::MAX; 4],
        })
    }

    fn release(&self, chunk: TerrainChunk) {
        let TerrainChunk {
            mesh_id,
            vertices_allocation,
            vertex_attributes_allocation,
            submeshes_allocation,
            ..
        } = chunk;

        let mesh_table = self.mesh_table.clone();
        let mesh_vertex = self.mesh_vertex.clone();
        let mesh_vertex_attribute = self.mesh_vertex_attribute.clone();

        self.deferred_destroy.push(move || {
            let erased = mesh_table.erase(mesh_id);

            mesh_table.submesh.allocator.release(submeshes_allocation);
            mesh_vertex.allocator.release(vertices_allocation);
            mesh_vertex_attribute.allocator.release(vertex_attributes_allocation);
            mesh_table.mesh.allocator.release(mesh_id);

            erased
        });
    }

    fn publish(&mut self, selected: &[ChunkCoordinate], observer: Vec3) {
        let limits = self.limits;

        let Self {
            chunks,
            mesh_table,
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

                mesh_id: chunk.mesh_id.inner,
                animation: None,
                outline: [0.0; 4],
            });

            chunk_views.push(TerrainChunkView {
                center,
                level: coordinate.level,

                mesh_id: chunk.mesh_id,
            });

            let level_deltas = ChunkSelection::level_deltas(*coordinate, observer, limits);

            if level_deltas == chunk.level_deltas {
                continue;
            }

            chunk.level_deltas = level_deltas;

            mesh_table.record_changed(chunk.mesh_id);

            stitch_requests.push(TerrainStitchRequest {
                mesh_id: chunk.mesh_id,
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
            if let Some(chunk) = self.chunks.remove(&coordinate) {
                self.release(chunk);
            }
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

impl Drop for Terrain {
    fn drop(&mut self) {
        for (_, chunk) in take(&mut self.chunks) {
            self.release(chunk);
        }

        let index = self.index.clone();
        let topology = self.topology;

        self.deferred_destroy.push(move || {
            index.allocator.release(topology);

            Ok(())
        });
    }
}
