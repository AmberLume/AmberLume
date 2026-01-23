use yakui::{column, pad, Color};
use yakui::widgets::{ColoredBox, Pad, Text};
use amber_lume::system_stats::SystemStats;
use amber_lume::ui::ui_fragment::UiFragment;

pub struct DebugFragment {
    pub fps: f32,
    pub cpu_frame_time: f32,
    pub gpu_frame_time: f32,
    pub total_frame_time: f32,
}

impl DebugFragment {
    pub fn collect_from(system_stats: &Option<SystemStats>) -> Self {
        let frame_stats = system_stats
            .and_then(|s| s.last_frame_stats);

        if let Some(frame_stats) = frame_stats {
            Self {
                fps: 1.0 / frame_stats.total_frame_time,
                cpu_frame_time: frame_stats.cpu_data_prepare_time * 1000.0,
                gpu_frame_time: frame_stats.gpu_render_time * 1000.0,
                total_frame_time: frame_stats.total_frame_time * 1000.0,
            }
        } else {
            Self {
                fps: 0.0,
                cpu_frame_time: 0.0,
                gpu_frame_time: 0.0,
                total_frame_time: 0.0,
            }
        }
    }

    fn draw_text(&self, value: String) {
        let mut text = Text::new(16.0, value);

        text.style.color = Color::WHITE;

        text.show();
    }
}

impl UiFragment for DebugFragment {
    fn render(&self) {
        let root = ColoredBox::container(Color::rgba(0, 0, 0, 127 + 64));

        root.show_children(|| {
            pad(Pad::all(12.0), || {
                column(|| {
                    self.draw_text(format!("FPS: {:.0}", self.fps));
                    self.draw_text(format!("CPU: {:.3}", self.cpu_frame_time));
                    self.draw_text(format!("GPU: {:.3}", self.gpu_frame_time));
                    self.draw_text(format!("Total: {:.3}", self.total_frame_time));
                });
            });
        });
    }
}
