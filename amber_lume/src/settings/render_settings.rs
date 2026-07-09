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

    pub rt_shadows: SwitchSetting,
    pub shadow_width: RangeSetting,
    pub shadow_softness: RangeSetting,
    pub shadow_samples: RangeSetting,

    pub rt_ao: SwitchSetting,
    pub ao_samples: RangeSetting,
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
            rt_shadows: SwitchSetting::new(
                false,
                false,
                "RT shadows",
                "Trace sun shadows against the ray-tracing acceleration structure instead of cascaded shadow maps. Requires ray-tracing support.",
            ),
            shadow_width: RangeSetting::new(
                0.02,
                0.02,
                0.0,
                0.5,
                "Shadow width",
                "World-space radius of the shadow penumbra (PCF kernel); larger softens and widens shadow edges, 0 = hard.",
            ),
            shadow_softness: RangeSetting::new(
                0.5,
                0.5,
                0.0,
                5.0,
                "Shadow softness",
                "Angular radius of the sun disk in degrees for ray-traced shadows; larger softens the penumbra, 0 = hard.",
            ),
            shadow_samples: RangeSetting::new(
                4.0,
                4.0,
                1.0,
                16.0,
                "Shadow samples",
                "Number of shadow rays traced per pixel for ray-traced shadows; higher is smoother but costlier.",
            ),
            rt_ao: SwitchSetting::new(
                false,
                false,
                "RT AO",
                "Trace ambient occlusion against the ray-tracing acceleration structure instead of screen-space GTAO. Requires ray-tracing support.",
            ),
            ao_samples: RangeSetting::new(
                4.0,
                4.0,
                1.0,
                16.0,
                "AO samples",
                "Number of occlusion rays traced per pixel for ray-traced ambient occlusion; higher is smoother but costlier.",
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
