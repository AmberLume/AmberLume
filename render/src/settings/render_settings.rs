use crate::settings::present_mode::PresentMode;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderSettings {
    pub present_mode: PresentMode,
    pub hdr: bool,
    pub render_scale: f32,
}
