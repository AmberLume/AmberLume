use crate::store::mesh_table::geometry_range::GeometryRange;
use index_allocator::ResourceId;

pub struct LoadedGeometry {
    pub mesh_id: ResourceId,

    pub ranges: Vec<GeometryRange>,
}
