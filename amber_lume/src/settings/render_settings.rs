use crate::settings::settings::RangeSetting;

#[derive(Copy, Clone)]
pub struct RenderSettings {
    pub exposure: RangeSetting,
}

impl Default for RenderSettings {
    fn default() -> Self {
        Self {
            exposure: RangeSetting::new(
                4.0,
                4.0,
                0.1,
                8.0,
                "Exposure",
                "Linear multiplier applied to the HDR scene before AgX tonemapping.",
            ),
        }
    }
}
