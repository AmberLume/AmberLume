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

    _pad0: [u32; 25],
}

impl AccumulatePushConstants {
    pub fn create(
        scene_color_texture: u32,
        velocity_texture: u32,
        history_prev_texture: u32,
        history_curr_storage: u32,
        history_valid: u32,
        display_width: u32,
        display_height: u32,
    ) -> Self {
        Self {
            scene_color_texture,
            velocity_texture,
            history_prev_texture,
            history_curr_storage,
            history_valid,
            display_width,
            display_height,

            _pad0: [0; 25],
        }
    }
}
