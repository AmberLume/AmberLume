use crate::hardware_capabilities::HardwareCapabilities;
use crate::present_mode::PresentMode;
use crate::settings::choice_setting::ChoiceSetting;
use crate::settings::range_setting::RangeSetting;
use crate::settings::switch_setting::SwitchSetting;


const DEBUG_LAYER_OPTIONS: &[&str] = &["Off", "Velocity", "Normal", "AO", "SH", "HiZ Near", "HiZ Far", "Shadow", "AO history", "AO denoised"];

#[derive(Copy, Clone)]
pub struct RenderSettings {
    pub debug_layer: ChoiceSetting,
    pub hiz_mip: RangeSetting,
    pub collider_rendering: SwitchSetting,

    pub terrain_freeze_observer: SwitchSetting,
    pub terrain_vertex_points: SwitchSetting,

    pub present_mode: ChoiceSetting,

    pub fsr_enabled: SwitchSetting,
    pub render_scale: RangeSetting,

    pub exposure: RangeSetting,
    pub hdr: SwitchSetting,
    pub paper_white: RangeSetting,
    pub bloom_intensity: RangeSetting,
    pub bloom_threshold: RangeSetting,

    pub sharpness: RangeSetting,

    pub shadow_enabled: SwitchSetting,
    pub transmissive_shadows: SwitchSetting,
    pub rt_shadows: SwitchSetting,
    pub shadow_softness: RangeSetting,
    pub shadow_samples: RangeSetting,
    pub shadow_denoise: SwitchSetting,

    pub ao_enabled: SwitchSetting,
    pub rt_ao: SwitchSetting,
    pub ao_spatial: SwitchSetting,
    pub gtao_radius: RangeSetting,
    pub gtao_power: RangeSetting,
    pub denoise_history: RangeSetting,
    pub ao_samples: RangeSetting,
}

impl Default for RenderSettings {
    fn default() -> Self {
        Self {
            debug_layer: ChoiceSetting::new(
                0,
                DEBUG_LAYER_OPTIONS,
                "Debug layer",
                "Render a selected intermediate render layer fullscreen instead of the final image.",
            ),
            hiz_mip: RangeSetting::new(
                0.0,
                0.0,
                15.0,
                "HiZ mip",
                "Which Hi-Z pyramid mip level to display.",
            ),
            collider_rendering: SwitchSetting::new(
                false,
                "Collider rendering enabled",
                "...",
            ),
            terrain_freeze_observer: SwitchSetting::new(
                false,
                "Freeze terrain observer",
                "Pin terrain streaming to the camera position captured when this was switched on, so the level of detail can be inspected from the outside.",
            ),
            terrain_vertex_points: SwitchSetting::new(
                false,
                "Terrain vertex points",
                "Draw a dot at every terrain vertex, coloured by its level of detail.",
            ),
            present_mode: ChoiceSetting::new(
                PresentMode::Mailbox.index(),
                PresentMode::OPTIONS,
                "VSync",
                "How finished frames reach the display: Immediate presents without waiting and can tear, Mailbox replaces the queued frame on every refresh, FIFO waits for the refresh and caps the frame rate. Unsupported modes fall back to FIFO.",
            ),
            fsr_enabled: SwitchSetting::new(
                true,
                "FSR",
                "Temporal upscaling and antialiasing (jitter + accumulation + sharpen). Off falls back to a plain bilinear upscale.",
            ),
            render_scale: RangeSetting::new(
                1.0,
                0.1,
                1.0,
                "Render scale",
                "Internal render resolution as a fraction of the display; the scene is rendered smaller and upscaled.",
            ),
            exposure: RangeSetting::new(
                4.0,
                0.1,
                8.0,
                "Exposure",
                "Linear multiplier applied to the HDR scene before AgX tonemapping.",
            ),
            hdr: SwitchSetting::new(
                false,
                "HDR",
                "Output to an HDR display via a scRGB swapchain. Only available when the surface supports it.",
            ),
            paper_white: RangeSetting::new(
                3.0,
                1.0,
                8.0,
                "HDR white",
                "SDR reference white level for HDR output, in units of scRGB white (1.0 = 80 nits).",
            ),
            bloom_intensity: RangeSetting::new(
                0.05,
                0.0,
                1.0,
                "Bloom",
                "Strength of the bloom glow added to the scene before tonemapping (0 disables it).",
            ),
            bloom_threshold: RangeSetting::new(
                1.0,
                0.0,
                4.0,
                "Bloom thr",
                "Brightness threshold above which pixels contribute to bloom.",
            ),
            sharpness: RangeSetting::new(
                0.5,
                0.0,
                1.0,
                "Sharpness",
                "RCAS sharpening strength applied to the upscaled image in tonemap (0 disables it).",
            ),
            shadow_enabled: SwitchSetting::new(
                true,
                "Shadow",
                "Sun shadows. Off makes the sun fully unshadowed and skips the shadow passes.",
            ),
            rt_shadows: SwitchSetting::new(
                false,
                "RT shadows",
                "Trace sun shadows against the ray-tracing acceleration structure instead of cascaded shadow maps. Requires ray-tracing support.",
            ),
            transmissive_shadows: SwitchSetting::new(
                true,
                "Transmissive shadows",
                "Trace shadow rays through blend materials and tint the light they pass.",
            ),
            shadow_softness: RangeSetting::new(
                0.5,
                0.0,
                5.0,
                "Shadow softness",
                "Angular radius of the sun disk in degrees for ray-traced shadows; larger softens the penumbra, 0 = hard.",
            ),
            shadow_samples: RangeSetting::new(
                4.0,
                1.0,
                16.0,
                "Shadow samples",
                "Number of shadow rays traced per pixel for ray-traced shadows; higher is smoother but costlier.",
            ),
            shadow_denoise: SwitchSetting::new(
                true,
                "Shadow denoise",
                "Temporal denoise of the ray-traced shadow so it can use fewer samples; adds a full-resolution pass. Off shows the raw traced shadow.",
            ),
            rt_ao: SwitchSetting::new(
                false,
                "RT AO",
                "Trace ambient occlusion against the ray-tracing acceleration structure instead of screen-space GTAO. Requires ray-tracing support.",
            ),
            ao_spatial: SwitchSetting::new(
                true,
                "AO spatial",
                "Edge-aware spatial filter applied to the traced occlusion before temporal accumulation; suppresses noise at the cost of a full-resolution pass.",
            ),
            ao_samples: RangeSetting::new(
                4.0,
                1.0,
                16.0,
                "AO samples",
                "Number of occlusion rays traced per pixel for ray-traced ambient occlusion; higher is smoother but costlier.",
            ),
            denoise_history: RangeSetting::new(
                16.0,
                1.0,
                64.0,
                "Denoise history",
                "Maximum frames the shadow and ambient occlusion denoiser accumulates over; higher is cleaner but lags more on change.",
            ),
            ao_enabled: SwitchSetting::new(
                true,
                "AO",
                "Ambient occlusion multiplied into the ambient term.",
            ),
            gtao_radius: RangeSetting::new(
                1.0,
                0.1,
                4.0,
                "AO radius",
                "World-space radius of the ambient occlusion search.",
            ),
            gtao_power: RangeSetting::new(
                1.5,
                0.5,
                4.0,
                "AO power",
                "Contrast applied to the ambient occlusion result (higher = darker occlusion).",
            ),
        }
    }
}

impl RenderSettings {
    pub fn with_hardware_defaults(mut self, capabilities: HardwareCapabilities) -> Self {
        self.rt_shadows.set(capabilities.ray_tracing);
        self.rt_ao.set(capabilities.ray_tracing);

        self
    }
}
