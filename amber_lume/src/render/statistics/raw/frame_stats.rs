#[derive(Debug, Copy, Clone)]
pub struct FrameStats {
    pub cpu_data_prepare_time: f32,
    pub gpu_render_time: f32,
    pub total_frame_time: f32,
}
