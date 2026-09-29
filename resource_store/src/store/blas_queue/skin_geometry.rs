use crate::store::blas_queue::geometry_range::GeometryRange;

#[derive(Clone)]
pub struct SkinGeometry {
    pub geometry_ranges: Vec<GeometryRange>,

    pub vertex_offset: u32,
    pub vertex_count: u32,
}
