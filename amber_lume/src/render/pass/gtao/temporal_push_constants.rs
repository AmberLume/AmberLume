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

    _pad0: [u32; 25],
}

impl GtaoTemporalPushConstants {
    pub fn create(
        gtao_texture: u32,
        velocity_texture: u32,
        history_prev_texture: u32,
        history_curr_storage: u32,
        history_valid: u32,
        width: u32,
        height: u32,
    ) -> Self {
        Self {
            gtao_texture,
            velocity_texture,
            history_prev_texture,
            history_curr_storage,
            history_valid,
            width,
            height,

            _pad0: [0; 25],
        }
    }
}
