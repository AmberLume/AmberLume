use bytemuck::{Pod, Zeroable};

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct SubmeshGPU {
    pub index_offset: u32,
    pub index_count: u32,
    pub vertex_offset: u32,
    pub vertex_attribute_offset: u32,
    pub vertex_skin_offset: u32,
    
    pub material_index: u32,
    _pad0: [u32; 2],
    
    pub bounds_min: [f32; 4],
    pub bounds_max: [f32; 4],
}

impl SubmeshGPU {
    pub fn create(
        index_count: u32,
        index_offset: u32,
        vertex_offset: u32,
        vertex_attribute_offset: u32,
        vertex_skin_offset: u32,
        material_index: u32,
        bounds: [f32; 6],
    ) -> Self {
        Self {
            index_offset,
            index_count,
            vertex_offset,
            vertex_attribute_offset,
            vertex_skin_offset,
            
            material_index,
            _pad0: [0; 2],
            
            bounds_min: [bounds[0], bounds[1], bounds[2], 0.0],
            bounds_max: [bounds[3], bounds[4], bounds[5], 0.0],
        }
    }
}
