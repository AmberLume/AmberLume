use std::fmt::Display;
use yakui::{checkbox, column, pad, text, Color, CrossAxisAlignment, MainAxisAlignment};
use yakui::widgets::{List, Pad, Text};
use amber_lume::amber_lume::AmberLume;
use amber_lume::resources::resource_indices_statistics::IndicesUsageStatistics;
use amber_lume::settings::settings::SwitchSetting;
use amber_lume::settings::settings_handler::EngineSettingsHandler;
use amber_lume::statistics::statistics_context::StatisticsSnapshot;
use amber_lume::ui::theme::Theme;
use amber_lume::ui::ui_state::UiFragmentState;
use crate::ui::widgets::tabs::tabs;

pub struct DebugFragmentState {
    pub statistics_snapshot: Option<StatisticsSnapshot>,
    pub statistics_smoothed: Option<StatisticsSnapshot>
}

impl DebugFragmentState {
    pub fn create() -> Self {
        Self {
            statistics_snapshot: None,
            statistics_smoothed: None,
        }
    }
}

impl UiFragmentState for DebugFragmentState {
    fn update(&mut self, amber_lume: &AmberLume) {
        let new_snapshot = amber_lume.statistics_snapshot();

        self.statistics_smoothed = Some(match &self.statistics_smoothed {
            Some(prev_smoothed) => prev_smoothed.smoothed(&new_snapshot, 0.01),
            None => new_snapshot,
        });
        self.statistics_snapshot = Some(new_snapshot);
    }

    fn render(&mut self, theme: &Theme, settings_handler: &EngineSettingsHandler) {
        tabs(&theme, &[
            ("Resource", &|| {
                pad(Pad::all(12.0), || {
                    column(|| {
                        if let Some(statistics_snapshot) = &self.statistics_smoothed {
                            usage_statistic("Indices", &statistics_snapshot.resource_indices.indices_used);
                            usage_statistic("Vertices", &statistics_snapshot.resource_indices.vertices_used);

                            usage_statistic("Textures", &statistics_snapshot.resource_indices.textures_used);
                            usage_statistic("Texture arrays", &statistics_snapshot.resource_indices.texture_arrays_used);
                            usage_statistic("Shadows", &statistics_snapshot.resource_indices.shadows_used);
                            usage_statistic("Shadow arrays", &statistics_snapshot.resource_indices.shadow_arrays_used);

                            usage_statistic("Pipelines", &statistics_snapshot.resource_indices.pipelines_used);
                            usage_statistic("Compute pipelines", &statistics_snapshot.resource_indices.compute_pipelines_used);
                        }
                    });
                });
            }),
            ("CPU", &|| {
                pad(Pad::all(12.0), || {
                    column(|| {
                        if let Some(statistics_snapshot) = &self.statistics_smoothed {
                            statistic_clipped("UI", &statistics_snapshot.cpu_render.ui_build.value);
                            statistic_clipped("Render commands", &statistics_snapshot.cpu_render.render_commands.value);
                        }
                    });
                });
            }),
            ("GPU", &|| {
                pad(Pad::all(12.0), || {
                    column(|| {
                        if let Some(statistics_snapshot) = &self.statistics_smoothed {
                            statistic_clipped("RenderPass", &statistics_snapshot.gpu_render.frame_time.value);

                            statistic_clipped("Rendered", &statistics_snapshot.gpu_render.submeshes_rendered);
                            statistic_clipped("Culled", &statistics_snapshot.gpu_render.submeshes_culled);
                        }
                    });
                });
            }),
            ("Physics", &|| {
                pad(Pad::all(12.0), || {
                    column(|| {
                        switch_option(settings_handler.get_pending().debug.collider_rendering_enabled, |new_value| {
                            settings_handler.update(|settings| {
                                settings.debug.collider_rendering_enabled.set(new_value);
                            })
                        });
                        switch_option(settings_handler.get_pending().debug.transform_interpolation, |new_value| {
                            settings_handler.update(|settings| {
                                settings.debug.transform_interpolation.set(new_value);
                            })
                        });
                    });
                });
            }),
        ], 0);
    }
}

fn statistic_clipped(title: &str, value: &dyn Display) {
    let mut text = Text::new(16.0, format!("{}: {:.3}", title, value));
    text.style.color = Color::WHITE;
    text.show();
}

fn usage_statistic(title: &str, usage: &Option<IndicesUsageStatistics>) {
    let value = if let Some(usage) = usage {
        let usage_percentage = (usage.used as f32 / usage.capacity as f32)  * 100.0;

        format!("{}/{} ({:.2}%) grave: {}", usage.used, usage.capacity, usage_percentage, usage.grave)
    } else {
        String::from("none")
    };

    pad(Pad::all(4.0), || {
        let mut text = Text::new(16.0, format!("{}: {}", title, value));
        text.style.color = Color::WHITE;
        text.show();
    });
}

fn switch_option(setting: SwitchSetting, on_change: impl FnOnce(bool)) {
    let value = format!("{}: ", setting.get_title());

    let mut row = List::row();
    row.main_axis_alignment = MainAxisAlignment::Start;
    row.cross_axis_alignment = CrossAxisAlignment::Center;

    row.show(|| {
        text(16.0, value);

        pad(Pad::all(4.0), || {
            let checkbox = checkbox(setting.get());

            if checkbox.checked != setting.get() {
                on_change(checkbox.checked);
            }
        });
    });
}
