use crate::terrain::terrain_chunk_view::TerrainChunkView;
use crate::terrain::terrain_frame::TerrainFrame;
use crate::terrain::terrain_generate_request::TerrainGenerateRequest;
use crate::terrain::terrain_stitch_request::TerrainStitchRequest;
use crate::terrain::terrain_chunk::TerrainChunk;
use anyhow::{bail, Context, Result};
use glam::{Mat4, Vec3};
use gpu::RangeAllocation;
use gpu::ResourceTransfer;
use gpu_data::SubmeshBoundsGPU;
use gpu_data::SubmeshGPU;
use gpu_data::VertexUvGPU;
use index_allocator::Allocation;
use index_allocator::DeferredDestroy;
use render_snapshot::{RenderEntity, RenderEntityId};
use resource_residency::ResRef;
use resource_store::{BlasEvent, BlasQueue, GeometryRange, MeshTable, VertexAllocation};
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
    blas_queue: Option<Arc<BlasQueue>>,

    index: Arc<RangeAllocation<u32>>,
    vertex: Arc<VertexAllocation>,
    vertex_uv: Arc<RangeAllocation<VertexUvGPU>>,
    submesh_bounds: Arc<RangeAllocation<SubmeshBoundsGPU>>,

    resource_transfer: Arc<ResourceTransfer>,
    deferred_destroy: Arc<DeferredDestroy>,

    source: ProceduralTerrainSource,
    limits: ResidencyLimits,

    anchor: Option<Vec3>,

    chunks: HashMap<ChunkCoordinate, TerrainChunk>,

    topology: Allocation,
    uvs: Allocation,

    generate_requests: Vec<TerrainGenerateRequest>,
    stitch_requests: Vec<TerrainStitchRequest>,
    chunk_views: Vec<TerrainChunkView>,
    drawables: Vec<RenderEntity>,
}

impl Terrain {
    pub fn new(
        mesh_table: Arc<MeshTable>,
        blas_queue: Option<Arc<BlasQueue>>,
        index: Arc<RangeAllocation<u32>>,
        vertex: Arc<VertexAllocation>,
        vertex_uv: Arc<RangeAllocation<VertexUvGPU>>,
        submesh_bounds: Arc<RangeAllocation<SubmeshBoundsGPU>>,
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

        let uvs_allocation = vertex_uv.allocator.allocate(ChunkGeometry::NODE_COUNT)
            .with_context(|| format!("Failed to allocate {} terrain uvs", ChunkGeometry::NODE_COUNT))?;

        let uvs = (0..ChunkGeometry::NODE_COUNT)
            .map(|node| {
                let column = node % ChunkGeometry::NODES;
                let row = node / ChunkGeometry::NODES;

                VertexUvGPU::new([
                    column as f32 / ChunkGeometry::CELLS as f32,
                    row as f32 / ChunkGeometry::CELLS as f32,
                ])
            })
            .collect::<Vec<_>>();

        resource_transfer.load_buffer_at(
            vertex_uv.slice(uvs_allocation.offset, uvs_allocation.size),
            &uvs,
        )?;

        Ok(Self {
            material,

            mesh_table,
            blas_queue,

            index,
            vertex,
            vertex_uv,
            submesh_bounds,

            resource_transfer,
            deferred_destroy,

            source: ProceduralTerrainSource::create(),
            limits: ResidencyLimits::create(),

            anchor: None,

            chunks: HashMap::new(),

            topology: topology_allocation,
            uvs: uvs_allocation,

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
        let vertices_allocation = self.vertex.allocator.allocate(ChunkGeometry::NODE_COUNT);
        let submeshes_allocation = self.mesh_table.submesh.allocator.allocate(1);
        let bounds_allocation = self.submesh_bounds.allocator.allocate(1);

        let (Some(mesh_id), Some(vertices_allocation), Some(submeshes_allocation), Some(bounds_allocation)) =
            (mesh_id, vertices_allocation, submeshes_allocation, bounds_allocation)
        else {
            if let Some(mesh_id) = mesh_id {
                self.mesh_table.mesh.allocator.release(mesh_id);
            }
            if let Some(vertices_allocation) = vertices_allocation {
                self.vertex.allocator.release(vertices_allocation);
            }
            if let Some(submeshes_allocation) = submeshes_allocation {
                self.mesh_table.submesh.allocator.release(submeshes_allocation);
            }
            if let Some(bounds_allocation) = bounds_allocation {
                self.submesh_bounds.allocator.release(bounds_allocation);
            }

            bail!("Resource buffers are full");
        };

        self.resource_transfer.load_buffer_at(
            self.submesh_bounds.slice(bounds_allocation.offset, bounds_allocation.size),
            &[SubmeshBoundsGPU::create(payload.bounds())],
        )?;

        let submesh = SubmeshGPU::create(
            self.topology.size,
            self.topology.offset,
            vertices_allocation.offset,
            self.uvs.offset,
            self.material.id.inner,
            bounds_allocation.offset,
        );

        self.mesh_table.write(
            mesh_id,
            submeshes_allocation,
            &[submesh],
            0,
            vertices_allocation.size,
            1,
        )?;

        if let Some(blas_queue) = &self.blas_queue {
            blas_queue.push(BlasEvent::Loaded {
                mesh_id,
                geometry_ranges: vec![GeometryRange {
                    index_count: self.topology.size,
                    index_offset: self.topology.offset,
                    vertex_offset: vertices_allocation.offset,
                    vertex_count: vertices_allocation.size,
                }],
                vertex_slice_stride: vertices_allocation.size,
                vertex_slice_count: 1,
            });
        }

        Ok(TerrainChunk {
            payload: Box::new(payload),

            mesh_id,
            vertices_allocation,
            submeshes_allocation,
            bounds_allocation,

            level_deltas: [u32::MAX; 4],
        })
    }

    fn release(&self, chunk: TerrainChunk) {
        let TerrainChunk {
            mesh_id,
            vertices_allocation,
            submeshes_allocation,
            bounds_allocation,
            ..
        } = chunk;

        let mesh_table = self.mesh_table.clone();
        let blas_queue = self.blas_queue.clone();
        let vertex = self.vertex.clone();
        let submesh_bounds = self.submesh_bounds.clone();

        self.deferred_destroy.push(move || {
            let erased = mesh_table.erase(mesh_id);

            if let Some(blas_queue) = &blas_queue {
                blas_queue.push(BlasEvent::Unloaded { mesh_id });
            }

            mesh_table.submesh.allocator.release(submeshes_allocation);
            vertex.allocator.release(vertices_allocation);
            submesh_bounds.allocator.release(bounds_allocation);
            mesh_table.mesh.allocator.release(mesh_id);

            erased
        });
    }

    fn publish(&mut self, selected: &[ChunkCoordinate], observer: Vec3) {
        let limits = self.limits;

        let Self {
            chunks,
            blas_queue,
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

            if let Some(blas_queue) = blas_queue {
                blas_queue.push(BlasEvent::Changed { mesh_id: chunk.mesh_id });
            }

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
        let vertex_uv = self.vertex_uv.clone();
        let topology = self.topology;
        let uvs = self.uvs;

        self.deferred_destroy.push(move || {
            index.allocator.release(topology);
            vertex_uv.allocator.release(uvs);

            Ok(())
        });
    }
}
