use crate::settings::{RangeSetting, SwitchSetting};

#[derive(Copy, Clone)]
pub struct LightSettings {
    pub auto_day_night: SwitchSetting,

    pub direction_x: RangeSetting,
    pub direction_y: RangeSetting,
    pub direction_z: RangeSetting,

    pub color_r: RangeSetting,
    pub color_g: RangeSetting,
    pub color_b: RangeSetting,

    pub intensity: RangeSetting,
    pub ibl_intensity: RangeSetting,
}

impl Default for LightSettings {
    fn default() -> Self {
        Self {
            auto_day_night: SwitchSetting::new(
                false,
                false,
                "Auto day/night",
                "Rotate the sun direction over time instead of using the manual direction sliders.",
            ),

            direction_x: RangeSetting::new(0.4, 0.4, -1.0, 1.0, "Dir X", "Sun direction X (normalized)."),
            direction_y: RangeSetting::new(-0.8, -0.8, -1.0, 1.0, "Dir Y", "Sun direction Y (normalized)."),
            direction_z: RangeSetting::new(0.3, 0.3, -1.0, 1.0, "Dir Z", "Sun direction Z (normalized)."),

            color_r: RangeSetting::new(1.0, 1.0, 0.0, 1.0, "Color R", "Sun color red channel."),
            color_g: RangeSetting::new(1.0, 1.0, 0.0, 1.0, "Color G", "Sun color green channel."),
            color_b: RangeSetting::new(1.0, 1.0, 0.0, 1.0, "Color B", "Sun color blue channel."),

            intensity: RangeSetting::new(1.0, 1.0, 0.0, 16.0, "Intensity", "Sun radiance multiplier."),
            ibl_intensity: RangeSetting::new(1.0, 1.0, 0.0, 2.0, "IBL Intensity", "Image-based lighting intensity multiplier."),
        }
    }
}
