use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct SkinTargetGPU {
    pub source_vertex_offset: u32,
    pub source_skin_offset: u32,
    pub target_vertex_offset: u32,
    pub skinning_instance_index: u32,
    pub first_vertex: u32,
}

impl SkinTargetGPU {
    pub fn new(
        source_vertex_offset: u32,
        source_skin_offset: u32,
        target_vertex_offset: u32,
        skinning_instance_index: u32,
        first_vertex: u32,
    ) -> Self {
        Self {
            source_vertex_offset,
            source_skin_offset,
            target_vertex_offset,
            skinning_instance_index,
            first_vertex,
        }
    }
}
