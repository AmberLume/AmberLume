use bytemuck::{Pod, Zeroable};

#[repr(C, align(4))]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct AccumulatePushConstants {
    pub scene_color_descriptor_id: u32,
    pub velocity_descriptor_id: u32,
    pub previous_history_descriptor_id: u32,
    pub current_history_storage_id: u32,
    pub history_valid: u32,
    pub display_width: u32,
    pub display_height: u32,

    _pad0: [u32; 25],
}

impl AccumulatePushConstants {
    pub fn create(
        scene_color_descriptor_id: u32,
        velocity_descriptor_id: u32,
        previous_history_descriptor_id: u32,
        current_history_storage_id: u32,
        history_valid: u32,
        display_width: u32,
        display_height: u32,
    ) -> Self {
        Self {
            scene_color_descriptor_id,
            velocity_descriptor_id,
            previous_history_descriptor_id,
            current_history_storage_id,
            history_valid,
            display_width,
            display_height,

            _pad0: [0; 25],
        }
    }
}
