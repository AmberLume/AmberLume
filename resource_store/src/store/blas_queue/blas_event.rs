use crate::store::blas_queue::geometry_range::GeometryRange;
use index_allocator::ResourceId;

pub enum BlasEvent {
    Loaded {
        mesh_id: ResourceId,
        geometry_ranges: Vec<GeometryRange>,
        vertex_slice_stride: u32,
        vertex_slice_count: u32,
    },
    Changed {
        mesh_id: ResourceId,
    },
    Unloaded {
        mesh_id: ResourceId,
    },
}
