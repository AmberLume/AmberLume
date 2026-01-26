use crate::system_stats::SystemStats;

pub trait UiRenderer {
    fn render(&self, system_stats: &Option<SystemStats>);
}
