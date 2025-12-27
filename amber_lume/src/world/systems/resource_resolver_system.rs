use crate::world::components::model_component::ModelComponent;
use crate::world::unique::resource_resolver_unique::ResourceResolverUnique;
use shipyard::{IntoIter, UniqueView, ViewMut};

pub fn resource_resolver_system(
    mut model_component: ViewMut<ModelComponent>,
    resource_resolver_unique: UniqueView<ResourceResolverUnique>,
) {
    let model_resolver = &resource_resolver_unique.model_provider;

    for model_component in (&mut model_component).iter() {
        model_resolver.ensure(&mut model_component.model_ref);
    }
}
