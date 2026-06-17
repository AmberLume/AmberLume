use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct GtaoTemporalPushConstants {
    pub gtao_texture: u32,
    pub velocity_texture: u32,
    pub history_prev_texture: u32,
    pub history_curr_storage: u32,
    pub history_valid: u32,
    pub width: u32,
    pub height: u32,
}
