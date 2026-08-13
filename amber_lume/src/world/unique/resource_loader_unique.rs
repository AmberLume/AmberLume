use std::sync::Arc;
use shipyard::Unique;
use resource_reader::ResourceReader;

#[derive(Unique)]
pub struct ResourceLoaderUnique {
    pub resource_reader: Arc<dyn ResourceReader>,
}

impl ResourceLoaderUnique {
    pub fn new(resource_reader: Arc<dyn ResourceReader>) -> Self {
        Self {
            resource_reader,
        }
    }
}
