use bytemuck::{Pod, Zeroable};

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct UiVertex {
    pub position: [f32; 2],
    pub tex_coords: [f32; 2],
    pub color: [f32; 4],
}
