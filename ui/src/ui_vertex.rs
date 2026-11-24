use bytemuck::{Pod, Zeroable};
use math::{Vec2, Vec4};

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct UiVertex {
    pub position: Vec2,
    pub tex_coords: Vec2,
    pub color: Vec4,
}
