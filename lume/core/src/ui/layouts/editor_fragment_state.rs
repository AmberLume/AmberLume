use yakui::Color;
use yakui::widgets::Text;
use settings::EngineSettingsHandler;
use ui::Theme;

pub struct EditorFragmentState {

}

impl EditorFragmentState {
    pub fn create() -> Self {
        Self {

        }
    }

    pub fn render(
        &mut self,
        _theme: &Theme,
        _settings_handler: &EngineSettingsHandler,
        picked_entity: Option<u32>,
    ) {
        let value = match picked_entity {
            Some(entity) => format!("Picked: {}", entity),
            None => String::from("Picked: -"),
        };

        let mut text = Text::new(16.0, value);
        text.style.color = Color::WHITE;
        text.show();
    }
}
