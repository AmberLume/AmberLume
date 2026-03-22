use crate::world::components::mesh_component::MeshComponent;
use crate::world::unique::resource_resolver_unique::ResourceResolverUnique;
use shipyard::{IntoIter, UniqueView, ViewMut};

pub fn resource_resolver_system(
    mut model_component: ViewMut<MeshComponent>,
    resource_resolver_unique: UniqueView<ResourceResolverUnique>,
) {
    let model_provider = &resource_resolver_unique.model_provider;

    for model_component in (&mut model_component).iter() {
        let res_ref = model_provider.get_or_load(model_component.config.clone());
        
        model_component.mesh_ref = Some(res_ref);
    }
}
