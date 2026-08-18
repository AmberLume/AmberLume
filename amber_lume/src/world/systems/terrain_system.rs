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
use index_allocator::Allocation;
use crate::render::frame_data::terrain_chunk_view::TerrainChunkView;
use crate::render::frame_data::terrain_generate_request::TerrainGenerateRequest;
use resource_residency::ResRef;
use resource_store::MeshConfig;
use shipyard::{AllStoragesViewMut, EntitiesViewMut, EntityId, Get, IntoIter, Remove, UniqueView, UniqueViewMut, View, ViewMut};
use std::collections::HashSet;
use std::sync::Arc;
use terrain::{ChunkCoordinate, ChunkGeometry, ChunkPayload, ChunkTopology, ResidencyUpdate, TerrainSource};
use tracing::error;

pub fn terrain_system(mut all_storages: AllStoragesViewMut) {
    let Some(update) = plan_residency(&all_storages) else {
        return;
    };

    let Some(shared_indices) = resolve_shared_indices(&all_storages) else {
        return;
    };

    let mut loaded = Vec::new();

    for coordinate in update.requested {
        if let Some(payload) = request_chunk(&all_storages, coordinate, shared_indices) {
            loaded.push(payload);
        }
    }

    for coordinate in plan_retired(&all_storages) {
        release_chunk(&mut all_storages, coordinate);
    }

    refresh_visibility(&all_storages);
    refresh_seams(&all_storages, loaded);

    collect_chunk_views(&all_storages);
}

fn collect_chunk_views(all_storages: &AllStoragesViewMut) {
    let Ok(mut terrain_unique) = all_storages.borrow::<UniqueViewMut<TerrainUnique>>() else {
        return;
    };

    let Ok(terrain_chunks) = all_storages.borrow::<View<TerrainChunkComponent>>() else {
        return;
    };

    let Ok(meshes) = all_storages.borrow::<View<MeshComponent>>() else {
        return;
    };

    terrain_unique.chunk_views = (&terrain_chunks, &meshes)
        .iter()
        .map(|(terrain_chunk, _)| TerrainChunkView {
            center: ChunkGeometry::chunk_center(terrain_chunk.coordinate),
            level: terrain_chunk.coordinate.level,

            vertex_offset: terrain_chunk.vertex_offset,
        })
        .collect();
}

fn plan_retired(all_storages: &AllStoragesViewMut) -> Vec<ChunkCoordinate> {
    let Some(observer) = observer_position(all_storages) else {
        return Vec::new();
    };

    let Ok(terrain_unique) = all_storages.borrow::<UniqueView<TerrainUnique>>() else {
        return Vec::new();
    };

    terrain_unique
        .residency
        .retired(observer, terrain_unique.limits)
}

fn refresh_seams(all_storages: &AllStoragesViewMut, loaded: Vec<ChunkPayload>) {
    let Ok(mut terrain_unique) = all_storages.borrow::<UniqueViewMut<TerrainUnique>>() else {
        return;
    };

    let Ok(mut terrain_chunks) = all_storages.borrow::<ViewMut<TerrainChunkComponent>>() else {
        return;
    };

    let max_level = terrain_unique.limits.max_level;

    let mut outdated = Vec::new();

    for terrain_chunk in (&mut terrain_chunks).iter() {
        let level_deltas = terrain_unique
            .residency
            .level_deltas(terrain_chunk.coordinate, max_level);

        let is_fresh = loaded
            .iter()
            .any(|payload| payload.coordinate() == terrain_chunk.coordinate);

        if level_deltas == terrain_chunk.level_deltas && !is_fresh {
            continue;
        }

        terrain_chunk.level_deltas = level_deltas;

        outdated.push((
            terrain_chunk.coordinate,
            terrain_chunk.vertex_offset,
            level_deltas,
        ));
    }

    for (coordinate, vertex_offset, level_deltas) in outdated {
        let heights = match loaded
            .iter()
            .find(|payload| payload.coordinate() == coordinate)
        {
            Some(payload) => payload.heights().to_vec(),
            None => match terrain_unique.source.load(coordinate) {
                Ok(payload) => payload.heights().to_vec(),
                Err(error) => {
                    error!("Failed to reload terrain chunk {coordinate:?}: {:#}", error);

                    continue;
                }
            },
        };

        terrain_unique.generate_requests.push(TerrainGenerateRequest {
            vertex_offset,
            cell_size: ChunkGeometry::cell_size(coordinate.level),
            level_deltas: pack_level_deltas(level_deltas),
            heights,
        });
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

fn plan_residency(all_storages: &AllStoragesViewMut) -> Option<ResidencyUpdate> {
    let render_view_unique = all_storages.borrow::<UniqueView<RenderViewUnique>>().ok()?;
    let settings_unique = all_storages.borrow::<UniqueView<SettingsUnique>>().ok()?;
    let mut terrain_unique = all_storages.borrow::<UniqueViewMut<TerrainUnique>>().ok()?;

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

    let limits = terrain_unique.limits;

    Some(terrain_unique.residency.update(observer, limits))
}

fn resolve_shared_indices(all_storages: &AllStoragesViewMut) -> Option<Allocation> {
    let mut terrain_unique = all_storages.borrow::<UniqueViewMut<TerrainUnique>>().ok()?;

    if let Some(shared_indices) = terrain_unique.shared_indices {
        return Some(shared_indices);
    }

    let resource_resolver_unique = all_storages.borrow::<UniqueView<ResourceResolverUnique>>().ok()?;

    let topology = ChunkTopology::build();

    match resource_resolver_unique.mesh_provider.backend.reserve_indices(topology.indices()) {
        Ok(shared_indices) => {
            terrain_unique.shared_indices = Some(shared_indices);

            Some(shared_indices)
        }
        Err(error) => {
            error!("Failed to reserve terrain indices: {:#}", error);

            None
        }
    }
}

fn release_chunk(all_storages: &mut AllStoragesViewMut, coordinate: ChunkCoordinate) {
    let entity_id = {
        let Ok(terrain_chunks) = all_storages.borrow::<View<TerrainChunkComponent>>() else {
            return;
        };

        terrain_chunks
            .iter()
            .with_id()
            .find(|(_, terrain_chunk)| terrain_chunk.coordinate == coordinate)
            .map(|(entity_id, _)| entity_id)
    };

    let Some(entity_id) = entity_id else {
        return;
    };

    all_storages.delete_entity(entity_id);

    if let Ok(mut terrain_unique) = all_storages.borrow::<UniqueViewMut<TerrainUnique>>() {
        terrain_unique.residency.mark_released(coordinate);
    }
}

fn request_chunk(
    all_storages: &AllStoragesViewMut,
    coordinate: ChunkCoordinate,
    shared_indices: Allocation,
) -> Option<ChunkPayload> {
    let resource_resolver_unique = all_storages.borrow::<UniqueView<ResourceResolverUnique>>().ok()?;
    let mut terrain_unique = all_storages.borrow::<UniqueViewMut<TerrainUnique>>().ok()?;

    let mesh_provider = &resource_resolver_unique.mesh_provider;

    let payload = match terrain_unique.source.load(coordinate) {
        Ok(payload) => payload,
        Err(error) => {
            error!("Failed to load terrain chunk {coordinate:?}: {:#}", error);

            return None;
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

            return None;
        }
    };

    let vertex_offset = mesh_provider.with_resource(handle.id, |mesh| mesh.vertices_allocation.offset);

    let Some(vertex_offset) = vertex_offset else {
        error!("Reserved terrain mesh {coordinate:?} is not available");

        return None;
    };

    spawn_chunk(all_storages, coordinate, handle, vertex_offset)?;

    terrain_unique.residency.mark_resident(coordinate);

    Some(payload)
}

fn spawn_chunk(
    all_storages: &AllStoragesViewMut,
    coordinate: ChunkCoordinate,
    handle: Arc<ResRef>,
    vertex_offset: u32,
) -> Option<EntityId> {
    let mut entities = all_storages.borrow::<EntitiesViewMut>().ok()?;
    let mut positions = all_storages.borrow::<ViewMut<PositionComponent>>().ok()?;
    let mut rotations = all_storages.borrow::<ViewMut<RotationComponent>>().ok()?;
    let mut scales = all_storages.borrow::<ViewMut<ScaleComponent>>().ok()?;
    let mut terrain_chunks = all_storages.borrow::<ViewMut<TerrainChunkComponent>>().ok()?;

    Some(entities.add_entity(
        (
            &mut positions,
            &mut rotations,
            &mut scales,
            &mut terrain_chunks,
        ),
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
            },
        ),
    ))
}

fn refresh_visibility(all_storages: &AllStoragesViewMut) {
    let visible = {
        let Some(observer) = observer_position(all_storages) else {
            return;
        };

        let Ok(mut terrain_unique) = all_storages.borrow::<UniqueViewMut<TerrainUnique>>() else {
            return;
        };

        let visible = terrain_unique
            .residency
            .visible(observer, terrain_unique.limits);

        terrain_unique.residency.publish_visible(&visible);

        visible.into_iter().collect::<HashSet<_>>()
    };

    let Ok(entities) = all_storages.borrow::<EntitiesViewMut>() else {
        return;
    };

    let Ok(terrain_chunks) = all_storages.borrow::<View<TerrainChunkComponent>>() else {
        return;
    };

    let Ok(mut meshes) = all_storages.borrow::<ViewMut<MeshComponent>>() else {
        return;
    };

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
}

fn observer_position(all_storages: &AllStoragesViewMut) -> Option<Vec3> {
    let terrain_unique = all_storages.borrow::<UniqueView<TerrainUnique>>().ok()?;

    if let Some(frozen) = terrain_unique.frozen_observer {
        return Some(frozen);
    }

    all_storages
        .borrow::<UniqueView<RenderViewUnique>>()
        .ok()
        .map(|render_view_unique| render_view_unique.resolved_camera.position)
}

fn chunk_key(coordinate: ChunkCoordinate) -> u64 {
    const COORDINATE_BITS: u32 = 28;
    const COORDINATE_MASK: u64 = (1 << COORDINATE_BITS) - 1;

    ((coordinate.level as u64) << (COORDINATE_BITS * 2))
        | ((coordinate.x as u32 as u64 & COORDINATE_MASK) << COORDINATE_BITS)
        | (coordinate.z as u32 as u64 & COORDINATE_MASK)
}
