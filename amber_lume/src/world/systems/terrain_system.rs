use crate::render::frame_data::terrain_chunk_view::TerrainChunkView;
use crate::render::frame_data::terrain_generate_request::TerrainGenerateRequest;
use crate::terrain::terrain_chunk::TerrainChunk;
use crate::world::components::mesh_component::MeshComponent;
use crate::world::components::position_component::PositionComponent;
use crate::world::components::rotation_component::RotationComponent;
use crate::world::components::scale_component::ScaleComponent;
use crate::world::unique::render_view_unique::RenderViewUnique;
use crate::world::unique::resource_resolver_unique::ResourceResolverUnique;
use crate::world::unique::settings_unique::SettingsUnique;
use crate::world::unique::terrain_unique::TerrainUnique;
use glam::{Quat, Vec3};
use gpu_data::SubmeshGPU;
use ray_tracing::BLASRequest;
use resource_store::MeshConfig;
use shipyard::{EntitiesViewMut, Get, Remove, UniqueView, UniqueViewMut, ViewMut};
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

    let TerrainUnique {
        material,
        blas_request_queue,
        source,
        residency,
        chunks,
        generate_requests,
        chunk_views,
        ..
    } = &mut *terrain_unique;

    let update = residency.update(observer, limits);

    let mut loaded = Vec::new();

    for coordinate in update.requested {
        let payload = match source.load(coordinate) {
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
            material: material.clone(),
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

        let entity = entities.add_entity(
            (&mut positions, &mut rotations, &mut scales),
            (
                PositionComponent {
                    position: ChunkGeometry::chunk_center(coordinate),
                },
                RotationComponent {
                    rotation: Quat::IDENTITY,
                },
                ScaleComponent { scale: Vec3::ONE },
            ),
        );

        chunks.insert(
            coordinate,
            TerrainChunk {
                entity,
                handle,
                vertex_offset,
                level_deltas: [u32::MAX; 4],
                traced: false,
            },
        );

        residency.mark_resident(coordinate);

        loaded.push(payload);
    }

    for coordinate in residency.retired(observer, limits) {
        let Some(chunk) = chunks.remove(&coordinate) else {
            continue;
        };

        positions.remove(chunk.entity);
        rotations.remove(chunk.entity);
        scales.remove(chunk.entity);
        meshes.remove(chunk.entity);

        entities.delete_unchecked(chunk.entity);

        residency.mark_released(coordinate);
    }

    let visible = residency.visible(observer, limits);

    residency.publish_visible(&visible);

    let visible = visible.into_iter().collect::<HashSet<_>>();

    for (coordinate, chunk) in chunks.iter() {
        let should_be_visible = visible.contains(coordinate);
        let is_visible = meshes.get(chunk.entity).is_ok();

        if should_be_visible == is_visible {
            continue;
        }

        if should_be_visible {
            entities.add_component(
                chunk.entity,
                &mut meshes,
                MeshComponent {
                    handle: chunk.handle.clone(),
                    skeleton: None,
                },
            );
        } else {
            meshes.remove(chunk.entity);
        }
    }

    for (coordinate, chunk) in chunks.iter_mut() {
        let coordinate = *coordinate;

        let level_deltas = residency.level_deltas(coordinate, limits.max_level);
        let traced =
            TerrainResidency::distance_to(observer, coordinate) <= limits.ray_tracing_distance;

        let is_fresh = loaded
            .iter()
            .any(|payload| payload.coordinate() == coordinate);

        if level_deltas == chunk.level_deltas && traced == chunk.traced && !is_fresh {
            continue;
        }

        if chunk.traced && !traced {
            blas_request_queue.push_unloaded(chunk.handle.id);
        }

        chunk.level_deltas = level_deltas;
        chunk.traced = traced;

        let reloaded;

        let payload = match loaded
            .iter()
            .find(|payload| payload.coordinate() == coordinate)
        {
            Some(payload) => payload,
            None => match source.load(coordinate) {
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

        generate_requests.push(TerrainGenerateRequest {
            vertex_offset: chunk.vertex_offset,
            cell_size: ChunkGeometry::cell_size(coordinate.level),
            level_deltas: pack_level_deltas(level_deltas),
            heights: payload.heights().to_vec(),
        });

        if traced {
            blas_request_queue.push_generated(BLASRequest {
                mesh_id: chunk.handle.id,
                submeshes: vec![SubmeshGPU::create(
                    shared_indices.size,
                    shared_indices.offset,
                    chunk.vertex_offset,
                    material.id.inner,
                    payload.bounds(),
                )],
            });
        }
    }

    *chunk_views = chunks
        .iter()
        .filter(|(_, chunk)| meshes.get(chunk.entity).is_ok())
        .map(|(coordinate, chunk)| TerrainChunkView {
            center: ChunkGeometry::chunk_center(*coordinate),
            level: coordinate.level,

            vertex_offset: chunk.vertex_offset,
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
