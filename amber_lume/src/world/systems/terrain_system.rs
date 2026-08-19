use crate::render::frame_data::terrain_chunk_view::TerrainChunkView;
use crate::render::frame_data::terrain_generate_request::TerrainGenerateRequest;
use crate::world::components::mesh_component::MeshComponent;
use crate::world::components::position_component::PositionComponent;
use crate::world::components::rotation_component::RotationComponent;
use crate::world::components::scale_component::ScaleComponent;
use crate::world::components::terrain_chunk_component::TerrainChunkComponent;
use crate::world::unique::render_view_unique::RenderViewUnique;
use crate::world::unique::resource_resolver_unique::ResourceResolverUnique;
use crate::world::unique::settings_unique::SettingsUnique;
use crate::world::unique::terrain_unique::TerrainUnique;
use glam::{Quat, Vec3};
use gpu_data::SubmeshGPU;
use ray_tracing::BLASRequest;
use resource_store::MeshConfig;
use shipyard::{EntitiesViewMut, Get, IntoIter, Remove, UniqueView, UniqueViewMut, ViewMut};
use std::collections::HashSet;
use terrain::{ChunkCoordinate, ChunkGeometry, ChunkTopology, TerrainResidency, TerrainSource};
use tracing::error;

pub fn terrain_system(
    mut entities: EntitiesViewMut,
    mut terrain_unique: UniqueViewMut<TerrainUnique>,
    render_view_unique: UniqueView<RenderViewUnique>,
    settings_unique: UniqueView<SettingsUnique>,
    resource_resolver_unique: UniqueView<ResourceResolverUnique>,
    mut positions: ViewMut<PositionComponent>,
    mut rotations: ViewMut<RotationComponent>,
    mut scales: ViewMut<ScaleComponent>,
    mut terrain_chunks: ViewMut<TerrainChunkComponent>,
    mut meshes: ViewMut<MeshComponent>,
) {
    let camera = render_view_unique.resolved_camera.position;
    let frozen = settings_unique
        .settings
        .load()
        .render
        .terrain_freeze_observer
        .value;

    let observer = if frozen {
        *terrain_unique.frozen_observer.get_or_insert(camera)
    } else {
        terrain_unique.frozen_observer = None;

        camera
    };

    let mesh_provider = &resource_resolver_unique.mesh_provider;

    let shared_indices = match terrain_unique.shared_indices {
        Some(shared_indices) => shared_indices,
        None => match mesh_provider.backend.reserve_indices(ChunkTopology::build().indices()) {
            Ok(shared_indices) => {
                terrain_unique.shared_indices = Some(shared_indices);

                shared_indices
            }
            Err(error) => {
                error!("Failed to reserve terrain indices: {:#}", error);

                return;
            }
        },
    };

    let limits = terrain_unique.limits;
    let update = terrain_unique.residency.update(observer, limits);

    let mut loaded = Vec::new();

    for coordinate in update.requested {
        let payload = match terrain_unique.source.load(coordinate) {
            Ok(payload) => payload,
            Err(error) => {
                error!("Failed to load terrain chunk {coordinate:?}: {:#}", error);

                continue;
            }
        };

        let mesh_config = MeshConfig::Reserved {
            key: chunk_key(coordinate),
            vertex_count: ChunkGeometry::NODE_COUNT,
            index_offset: shared_indices.offset,
            index_count: shared_indices.size,
            material: terrain_unique.material.clone(),
            bounds: payload.bounds(),
        };

        let handle = match mesh_provider.acquire_sync(mesh_config) {
            Ok(handle) => handle,
            Err(error) => {
                error!("Failed to reserve terrain mesh {coordinate:?}: {:#}", error);

                continue;
            }
        };

        let vertex_offset = mesh_provider.with_resource(handle.id, |mesh| 
            mesh.vertices_allocation.offset
        );
        let Some(vertex_offset) = vertex_offset else {
            error!("Reserved terrain mesh {coordinate:?} is not available");

            continue;
        };

        entities.add_entity(
            (&mut positions, &mut rotations, &mut scales, &mut terrain_chunks),
            (
                PositionComponent {
                    position: ChunkGeometry::chunk_center(coordinate),
                },
                RotationComponent {
                    rotation: Quat::IDENTITY,
                },
                ScaleComponent { scale: Vec3::ONE },
                TerrainChunkComponent {
                    coordinate,
                    handle,
                    vertex_offset,
                    level_deltas: [u32::MAX; 4],
                    traced: false,
                },
            ),
        );

        terrain_unique.residency.mark_resident(coordinate);

        loaded.push(payload);
    }

    let retired = terrain_unique.residency.retired(observer, limits);

    for coordinate in retired {
        let entity_id = terrain_chunks
            .iter()
            .with_id()
            .find(|(_, terrain_chunk)| terrain_chunk.coordinate == coordinate)
            .map(|(entity_id, _)| entity_id);

        let Some(entity_id) = entity_id else {
            continue;
        };

        positions.remove(entity_id);
        rotations.remove(entity_id);
        scales.remove(entity_id);
        terrain_chunks.remove(entity_id);
        meshes.remove(entity_id);

        entities.delete_unchecked(entity_id);

        terrain_unique.residency.mark_released(coordinate);
    }

    let visible = terrain_unique.residency.visible(observer, limits);

    terrain_unique.residency.publish_visible(&visible);

    let visible = visible.into_iter().collect::<HashSet<_>>();

    for (entity_id, terrain_chunk) in terrain_chunks.iter().with_id() {
        let should_be_visible = visible.contains(&terrain_chunk.coordinate);
        let is_visible = meshes.get(entity_id).is_ok();

        if should_be_visible == is_visible {
            continue;
        }

        if should_be_visible {
            entities.add_component(
                entity_id,
                &mut meshes,
                MeshComponent {
                    handle: terrain_chunk.handle.clone(),
                    skeleton: None,
                },
            );
        } else {
            meshes.remove(entity_id);
        }
    }

    let max_level = terrain_unique.limits.max_level;

    let mut outdated = Vec::new();

    for terrain_chunk in (&mut terrain_chunks).iter() {
        let level_deltas = terrain_unique
            .residency
            .level_deltas(terrain_chunk.coordinate, max_level);

        let is_fresh = loaded
            .iter()
            .any(|payload| payload.coordinate() == terrain_chunk.coordinate);

        let traced = TerrainResidency::distance_to(observer, terrain_chunk.coordinate)
            <= limits.ray_tracing_distance;

        if level_deltas == terrain_chunk.level_deltas && traced == terrain_chunk.traced && !is_fresh
        {
            continue;
        }

        if terrain_chunk.traced && !traced {
            terrain_unique
                .blas_request_queue
                .push_unloaded(terrain_chunk.handle.id);
        }

        terrain_chunk.level_deltas = level_deltas;
        terrain_chunk.traced = traced;

        outdated.push((
            terrain_chunk.coordinate,
            terrain_chunk.vertex_offset,
            level_deltas,
            terrain_chunk.handle.id,
            traced,
        ));
    }

    for (coordinate, vertex_offset, level_deltas, mesh_id, traced) in outdated {
        let reloaded;

        let payload = match loaded
            .iter()
            .find(|payload| payload.coordinate() == coordinate)
        {
            Some(payload) => payload,
            None => match terrain_unique.source.load(coordinate) {
                Ok(payload) => {
                    reloaded = payload;

                    &reloaded
                }
                Err(error) => {
                    error!("Failed to reload terrain chunk {coordinate:?}: {:#}", error);

                    continue;
                }
            },
        };

        let submesh = SubmeshGPU::create(
            shared_indices.size,
            shared_indices.offset,
            vertex_offset,
            terrain_unique.material.id.inner,
            payload.bounds(),
        );

        terrain_unique.generate_requests.push(TerrainGenerateRequest {
            vertex_offset,
            cell_size: ChunkGeometry::cell_size(coordinate.level),
            level_deltas: pack_level_deltas(level_deltas),
            heights: payload.heights().to_vec(),
        });

        if traced {
            terrain_unique.blas_request_queue.push_generated(BLASRequest {
                mesh_id,
                submeshes: vec![submesh],
            });
        }
    }

    terrain_unique.chunk_views = (&terrain_chunks, &meshes)
        .iter()
        .map(|(terrain_chunk, _)| TerrainChunkView {
            center: ChunkGeometry::chunk_center(terrain_chunk.coordinate),
            level: terrain_chunk.coordinate.level,

            vertex_offset: terrain_chunk.vertex_offset,
        })
        .collect();
}

fn pack_level_deltas(level_deltas: [u32; 4]) -> u32 {
    level_deltas
        .iter()
        .enumerate()
        .fold(0, |packed, (side, delta)| {
            packed | ((delta & 0xFF) << (side * 8))
        })
}

fn chunk_key(coordinate: ChunkCoordinate) -> u64 {
    const COORDINATE_BITS: u32 = 28;
    const COORDINATE_MASK: u64 = (1 << COORDINATE_BITS) - 1;

    ((coordinate.level as u64) << (COORDINATE_BITS * 2))
        | ((coordinate.x as u32 as u64 & COORDINATE_MASK) << COORDINATE_BITS)
        | (coordinate.z as u32 as u64 & COORDINATE_MASK)
}
