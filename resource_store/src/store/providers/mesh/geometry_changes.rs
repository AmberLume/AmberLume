use crate::store::providers::mesh::loaded_geometry::LoadedGeometry;
use index_allocator::ResourceId;

#[derive(Default)]
pub struct GeometryChanges {
    pub loaded: Vec<LoadedGeometry>,
    pub changed: Vec<ResourceId>,
    pub unloaded: Vec<ResourceId>,
}
