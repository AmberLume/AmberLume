use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct AccumulatePushConstants {
    pub scene_color_texture: u32,
    pub velocity_texture: u32,
    pub history_prev_texture: u32,
    pub history_curr_storage: u32,
    pub history_valid: u32,
    pub display_width: u32,
    pub display_height: u32,
}
