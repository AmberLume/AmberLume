#[derive(Copy, Clone)]
pub struct ChoiceSetting {
    pub value: usize,

    pub options: &'static [&'static str],

    pub title: &'static str,
    pub description: &'static str,
}

impl ChoiceSetting {
    pub fn new(value: usize, options: &'static [&'static str], title: &'static str, description: &'static str) -> Self {
        Self {
            value,

            options,

            title,
            description,
        }
    }

    pub fn set(&mut self, value: usize) {
        if value < self.options.len() {
            self.value = value;
        }
    }
}
