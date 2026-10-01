use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct SubmeshGPU {
    pub index_offset: u32,
    pub index_count: u32,
    pub vertex_offset: u32,
    pub uv_offset: u32,
    pub material_index: u32,
    pub bounds_index: u32,
}

impl SubmeshGPU {
    pub fn create(
        index_count: u32,
        index_offset: u32,
        vertex_offset: u32,
        uv_offset: u32,
        material_index: u32,
        bounds_index: u32,
    ) -> Self {
        Self {
            index_offset,
            index_count,
            vertex_offset,
            uv_offset,
            material_index,
            bounds_index,
        }
    }
}
