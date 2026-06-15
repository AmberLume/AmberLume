use crate::settings::settings::{RangeSetting, SwitchSetting};

#[derive(Copy, Clone)]
pub struct RenderSettings {
    pub fsr_enabled: SwitchSetting,
    pub render_scale: RangeSetting,

    pub exposure: RangeSetting,
    pub hdr: SwitchSetting,
    pub paper_white: RangeSetting,
    pub bloom_intensity: RangeSetting,
    pub bloom_threshold: RangeSetting,

    pub sharpness: RangeSetting,

    pub gtao_enabled: SwitchSetting,
    pub gtao_radius: RangeSetting,
    pub gtao_power: RangeSetting,
}

impl Default for RenderSettings {
    fn default() -> Self {
        Self {
            fsr_enabled: SwitchSetting::new(
                true,
                true,
                "FSR",
                "Temporal upscaling and antialiasing (jitter + accumulation + sharpen). Off falls back to a plain bilinear upscale.",
            ),
            render_scale: RangeSetting::new(
                1.0,
                1.0,
                0.1,
                1.0,
                "Render scale",
                "Internal render resolution as a fraction of the display; the scene is rendered smaller and upscaled.",
            ),
            exposure: RangeSetting::new(
                4.0,
                4.0,
                0.1,
                8.0,
                "Exposure",
                "Linear multiplier applied to the HDR scene before AgX tonemapping.",
            ),
            hdr: SwitchSetting::new(
                false,
                false,
                "HDR",
                "Output to an HDR display via a scRGB swapchain. Only available when the surface supports it.",
            ),
            paper_white: RangeSetting::new(
                3.0,
                3.0,
                1.0,
                8.0,
                "HDR white",
                "SDR reference white level for HDR output, in units of scRGB white (1.0 = 80 nits).",
            ),
            bloom_intensity: RangeSetting::new(
                0.05,
                0.05,
                0.0,
                1.0,
                "Bloom",
                "Strength of the bloom glow added to the scene before tonemapping (0 disables it).",
            ),
            bloom_threshold: RangeSetting::new(
                1.0,
                1.0,
                0.0,
                4.0,
                "Bloom thr",
                "Brightness threshold above which pixels contribute to bloom.",
            ),
            sharpness: RangeSetting::new(
                0.5,
                0.5,
                0.0,
                1.0,
                "Sharpness",
                "RCAS sharpening strength applied to the upscaled image in tonemap (0 disables it).",
            ),
            gtao_enabled: SwitchSetting::new(
                true,
                true,
                "GTAO",
                "Ground-truth ambient occlusion multiplied into the ambient term.",
            ),
            gtao_radius: RangeSetting::new(
                1.0,
                1.0,
                0.1,
                4.0,
                "GTAO radius",
                "World-space radius of the GTAO occlusion search.",
            ),
            gtao_power: RangeSetting::new(
                1.5,
                1.5,
                0.5,
                4.0,
                "GTAO power",
                "Contrast applied to the GTAO result (higher = darker occlusion).",
            ),
        }
    }
}
