use crate::store::blas_queue::geometry_range::GeometryRange;
use gpu_data::SubmeshGPU;
use index_allocator::Allocation;

pub(crate) struct SkinSource {
    pub vertices_allocation: Allocation,
    pub vertex_skins_allocation: Allocation,
    pub bone_offset: u32,

    pub submeshes: Vec<SubmeshGPU>,
    pub geometry_ranges: Vec<GeometryRange>,
}
