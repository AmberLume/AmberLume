use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct TemporalDenoisePushConstants {
    pub guide_curr_tex: u32,
    pub guide_prev_tex: u32,
    pub velocity_tex: u32,
    pub noisy_tex: u32,
    pub signal_prev_tex: u32,
    pub signal_storage: u32,

    pub width: u32,
    pub height: u32,
    pub history_valid: u32,
    pub history_frames: u32,
    pub frame_number: u32,
    pub trace_period: u32,
    pub variance_clamp: u32,
    pub colored: u32,

    pub tau_z: f32,
    pub tau_n: f32,

    _pad0: [u32; 16],
}

impl TemporalDenoisePushConstants {
    pub fn create(
        guide_curr_tex: u32,
        guide_prev_tex: u32,
        velocity_tex: u32,
        noisy_tex: u32,
        signal_prev_tex: u32,
        signal_storage: u32,
        width: u32,
        height: u32,
        history_valid: u32,
        history_frames: u32,
        frame_number: u32,
        trace_period: u32,
        variance_clamp: u32,
        colored: u32,
        tau_z: f32,
        tau_n: f32,
    ) -> Self {
        Self {
            guide_curr_tex,
            guide_prev_tex,
            velocity_tex,
            noisy_tex,
            signal_prev_tex,
            signal_storage,

            width,
            height,
            history_valid,
            history_frames,
            frame_number,
            trace_period,
            variance_clamp,
            colored,

            tau_z,
            tau_n,

            _pad0: [0; 16],
        }
    }
}
