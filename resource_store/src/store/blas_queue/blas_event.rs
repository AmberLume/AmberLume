use crate::store::blas_queue::geometry_range::GeometryRange;
use crate::store::blas_queue::skin_geometry::SkinGeometry;
use index_allocator::ResourceId;

pub enum BlasEvent {
    Loaded {
        mesh_id: ResourceId,
        geometry_ranges: Vec<GeometryRange>,
    },
    Changed {
        mesh_id: ResourceId,
    },
    Unloaded {
        mesh_id: ResourceId,
    },
    SkinLoaded {
        skin_id: ResourceId,
        geometry: SkinGeometry,
    },
    SkinUnloaded {
        skin_id: ResourceId,
    },
}
