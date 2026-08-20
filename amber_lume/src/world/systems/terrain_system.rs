use crate::world::unique::render_view_unique::RenderViewUnique;
use crate::world::unique::resource_resolver_unique::ResourceResolverUnique;
use crate::world::unique::settings_unique::SettingsUnique;
use crate::world::unique::terrain_unique::TerrainUnique;
use shipyard::{UniqueView, UniqueViewMut};

pub fn terrain_system(
    mut terrain_unique: UniqueViewMut<TerrainUnique>,
    render_view_unique: UniqueView<RenderViewUnique>,
    settings_unique: UniqueView<SettingsUnique>,
    resource_resolver_unique: UniqueView<ResourceResolverUnique>,
) {
    let freeze_observer = settings_unique
        .settings
        .load()
        .render
        .terrain_freeze_observer
        .value;

    let observer = terrain_unique.observer(render_view_unique.resolved_camera.position, freeze_observer);

    terrain_unique.terrain.chunks_for(observer, &resource_resolver_unique.mesh_provider);
}
