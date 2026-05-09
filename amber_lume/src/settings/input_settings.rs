use crate::settings::settings::SwitchSetting;

#[derive(Copy, Clone)]
pub struct InputSettings {
    pub cursor_controls_camera: SwitchSetting,
}

impl Default for InputSettings {
    fn default() -> Self {
        Self {
            cursor_controls_camera: SwitchSetting::new(
                true,
                true,
                "Cursor controls camera",
                "When enabled, mouse motion rotates the camera and the cursor is grabbed and hidden. When disabled, the cursor is free and can interact with UI.",
            ),
        }
    }
}
