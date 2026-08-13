#[derive(Copy, Clone)]
pub struct SwitchSetting {
    pub value: bool,

    pub title: &'static str,
    pub description: &'static str,
}

impl SwitchSetting {
    pub fn new(value: bool, title: &'static str, description: &'static str) -> Self {
        Self {
            value,

            title,
            description,
        }
    }

    pub fn set(&mut self, value: bool) {
        self.value = value;
    }
}
