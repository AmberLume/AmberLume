use amber_lume::system_stats::SystemStats;
use amber_lume::ui::ui_fragment::UiFragment;
use amber_lume::ui::ui_renderer::UiRenderer;
use crate::ui::layouts::root_layout::RootFragment;

pub struct LumeUiRenderer;

impl LumeUiRenderer {
    pub fn new() -> Self {
        Self { }
    }
}

impl UiRenderer for LumeUiRenderer {
    fn render(&self, system_stats: &Option<SystemStats>) {
        let root_fragment = RootFragment::collect_from(&system_stats);

        root_fragment.render();
    }
}
