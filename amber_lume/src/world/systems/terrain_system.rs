use crate::world::components::mesh_component::MeshComponent;
use crate::world::components::position_component::PositionComponent;
use crate::world::components::rotation_component::RotationComponent;
use crate::world::components::scale_component::ScaleComponent;
use crate::world::components::terrain_chunk_component::TerrainChunkComponent;
use crate::world::unique::render_view_unique::RenderViewUnique;
use crate::world::unique::resource_resolver_unique::ResourceResolverUnique;
use crate::world::unique::terrain_unique::TerrainUnique;
use glam::{Quat, Vec3};
use index_allocator::Allocation;
use crate::render::frame_data::terrain_generate_request::TerrainGenerateRequest;
use resource_residency::ResRef;
use resource_store::MeshConfig;
use shipyard::{AllStoragesViewMut, EntitiesViewMut, EntityId, IntoIter, UniqueView, UniqueViewMut, View, ViewMut};
use std::sync::Arc;
use terrain::{ChunkCoordinate, ChunkGeometry, ChunkTopology, ResidencyUpdate, TerrainSource};
use tracing::error;

pub fn terrain_system(mut all_storages: AllStoragesViewMut) {
    let Some(update) = plan_residency(&all_storages) else {
        return;
    };

    for coordinate in update.retired {
        release_chunk(&mut all_storages, coordinate);
    }

    let Some(shared_indices) = resolve_shared_indices(&all_storages) else {
        return;
    };

    for coordinate in update.requested {
        request_chunk(&all_storages, coordinate, shared_indices);
    }
}

fn plan_residency(all_storages: &AllStoragesViewMut) -> Option<ResidencyUpdate> {
    let render_view_unique = all_storages.borrow::<UniqueView<RenderViewUnique>>().ok()?;
    let terrain_unique = all_storages.borrow::<UniqueView<TerrainUnique>>().ok()?;

    Some(terrain_unique.residency.update(
        render_view_unique.resolved_camera.position,
        terrain_unique.load_distance,
    ))
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
) {
    let Ok(resource_resolver_unique) = all_storages.borrow::<UniqueView<ResourceResolverUnique>>() else {
        return;
    };

    let Ok(mut terrain_unique) = all_storages.borrow::<UniqueViewMut<TerrainUnique>>() else {
        return;
    };

    let mesh_provider = &resource_resolver_unique.mesh_provider;

    let payload = match terrain_unique.source.load(coordinate) {
        Ok(payload) => payload,
        Err(error) => {
            error!("Failed to load terrain chunk {coordinate:?}: {:#}", error);

            return;
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

            return;
        }
    };

    let vertex_offset = mesh_provider.with_resource(handle.id, |mesh| mesh.vertices_allocation.offset);

    let Some(vertex_offset) = vertex_offset else {
        error!("Reserved terrain mesh {coordinate:?} is not available");

        return;
    };

    if spawn_chunk(all_storages, coordinate, handle, vertex_offset).is_none() {
        return;
    }

    terrain_unique.generate_requests.push(TerrainGenerateRequest {
        vertex_offset,
        heights: payload.heights().to_vec(),
    });

    terrain_unique.residency.mark_resident(coordinate);
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
    let mut meshes = all_storages.borrow::<ViewMut<MeshComponent>>().ok()?;
    let mut terrain_chunks = all_storages.borrow::<ViewMut<TerrainChunkComponent>>().ok()?;

    Some(entities.add_entity(
        (
            &mut positions,
            &mut rotations,
            &mut scales,
            &mut meshes,
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
            MeshComponent {
                handle,
                skeleton: None,
            },
            TerrainChunkComponent {
                coordinate,
                vertex_offset,
            },
        ),
    ))
}

fn chunk_key(coordinate: ChunkCoordinate) -> u64 {
    ((coordinate.x as u32 as u64) << 32) | coordinate.z as u32 as u64
}
