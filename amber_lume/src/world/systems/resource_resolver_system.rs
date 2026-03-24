use crate::world::components::mesh_component::MeshComponent;
use crate::world::unique::resource_resolver_unique::ResourceResolverUnique;
use shipyard::{IntoIter, UniqueView, ViewMut};

pub fn resource_resolver_system(
    mut mesh_components: ViewMut<MeshComponent>,
    resource_resolver_unique: UniqueView<ResourceResolverUnique>,
) {
    let mesh_provider = &resource_resolver_unique.mesh_provider;

    for mesh_component in (&mut mesh_components).iter() {
        let res_ref = mesh_provider.get_or_load(mesh_component.config.clone());
        
        mesh_component.mesh_ref = Some(res_ref);
    }
}
