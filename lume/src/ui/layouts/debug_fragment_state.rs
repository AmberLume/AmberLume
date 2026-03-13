use yakui::{column, pad, Color};
use yakui::widgets::{Pad, Text};
use amber_lume::amber_lume::AmberLume;
use amber_lume::ui::ui_context::UiContext;
use amber_lume::ui::ui_state::UiFragmentState;

pub struct DebugFragmentState {
    pub fps: f32,
    pub cpu_frame_time: f32,
    pub gpu_frame_time: f32,
    pub ecs_frame_time: Option<f32>,
    pub total_frame_time: f32,

    pub entities_ecs: u32,
    pub submeshes_rendered: u32,
    pub submeshes_culled: u32,
}

impl DebugFragmentState {
    pub fn create() -> Self {
        Self {
            fps: 0.0,
            cpu_frame_time: 0.0,
            gpu_frame_time: 0.0,
            ecs_frame_time: None,
            total_frame_time: 0.0,

            entities_ecs: 0,

            submeshes_rendered: 0,
            submeshes_culled: 0,
        }
    }

    fn draw_text(&self, value: String) {
        let mut text = Text::new(16.0, value);

        text.style.color = Color::WHITE;

        text.show();
    }
}

impl UiFragmentState for DebugFragmentState {
    fn update(&mut self, amber_lume: &AmberLume) {
        if let Some(system_stats) = amber_lume.system_stats_handler.get_snapshot() {
            self.ecs_frame_time = system_stats.world_iteration_time.map(|t| t * 1000.0);

            self.entities_ecs = system_stats.entities_ecs;
            self.submeshes_rendered = system_stats.submeshes_rendered;
            self.submeshes_culled = system_stats.submeshes_culled;

            if let Some(frame_stats) = system_stats.last_frame_stats {
                self.fps = 1.0 / frame_stats.total_frame_time;
                self.cpu_frame_time = frame_stats.cpu_data_prepare_time * 1000.0;
                self.gpu_frame_time = frame_stats.gpu_render_time * 1000.0;

                self.total_frame_time = frame_stats.total_frame_time * 1000.0;
            }
        }
    }

    fn render(&mut self, _context: &UiContext) {
        pad(Pad::all(12.0), || {
            column(|| {
                self.draw_text(format!("FPS: {:.0}", self.fps));
                self.draw_text(format!("CPU: {:.3}", self.cpu_frame_time));
                self.draw_text(format!("GPU: {:.3}", self.gpu_frame_time));
                self.draw_text(
                    if let Some(world_frame_time) = self.ecs_frame_time {
                        format!("World: {:.3}", world_frame_time)
                    } else {
                        "World: -".to_owned()
                    }
                );
                self.draw_text(format!("Total: {:.3}", self.total_frame_time));
                self.draw_text(" ".to_string());
                self.draw_text("Entities:".to_string());
                self.draw_text(format!("    ECS: {}", self.entities_ecs));
                self.draw_text(" ".to_string());
                self.draw_text("Submeshes:".to_string());
                self.draw_text(format!("    Rendered: {}", self.submeshes_rendered));
                self.draw_text(format!("    Culled: {}", self.submeshes_culled));
            });
        });
    }
}
