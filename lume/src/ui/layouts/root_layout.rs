use yakui::{align, constrained, Alignment, Constraints, Vec2};
use amber_lume::system_stats::SystemStats;
use amber_lume::ui::ui_fragment::UiFragment;
use crate::ui::layouts::debug_layout::DebugFragment;

pub struct RootFragment {
    pub debug_fragment: DebugFragment,
}

impl RootFragment {
    pub fn collect_from(
        system_stats: &Option<SystemStats>,
    ) -> Self {
        let debug_fragment = DebugFragment::collect_from(system_stats);

        Self {
            debug_fragment,
        }
    }
}

impl UiFragment for RootFragment {
    fn render(&self) {
        align(Alignment::TOP_LEFT, || {
            let constraints = Constraints::tight(Vec2::new(300.0, 100.0));
            
            constrained(constraints, || {
                self.debug_fragment.render();
            });
        });
    }
}
