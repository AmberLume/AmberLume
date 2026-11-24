use bytemuck::{Pod, Zeroable};

#[repr(C, align(4))]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct TemporalDenoisePushConstants {
    pub current_guide_descriptor_id: u32,
    pub previous_guide_descriptor_id: u32,
    pub velocity_descriptor_id: u32,
    pub noisy_descriptor_id: u32,
    pub previous_signal_descriptor_id: u32,
    pub signal_storage_id: u32,

    pub width: u32,
    pub height: u32,
    pub history_valid: u32,
    pub history_frames: u32,
    pub frame_number: u32,
    pub colored: u32,

    pub depth_threshold: f32,
    pub normal_threshold: f32,

    _pad0: [u32; 18],
}

impl TemporalDenoisePushConstants {
    pub fn create(
        current_guide_descriptor_id: u32,
        previous_guide_descriptor_id: u32,
        velocity_descriptor_id: u32,
        noisy_descriptor_id: u32,
        previous_signal_descriptor_id: u32,
        signal_storage_id: u32,
        width: u32,
        height: u32,
        history_valid: u32,
        history_frames: u32,
        frame_number: u32,
        colored: u32,
        depth_threshold: f32,
        normal_threshold: f32,
    ) -> Self {
        Self {
            current_guide_descriptor_id,
            previous_guide_descriptor_id,
            velocity_descriptor_id,
            noisy_descriptor_id,
            previous_signal_descriptor_id,
            signal_storage_id,

            width,
            height,
            history_valid,
            history_frames,
            frame_number,
            colored,

            depth_threshold,
            normal_threshold,

            _pad0: [0; 18],
        }
    }
}
