use yakui::{button, checkbox, column, pad, text, Color, CrossAxisAlignment, MainAxisAlignment};
use yakui::widgets::{List, Pad, Text};
use amber_lume::render::pass::culling_indirect::render_view_culling_indirect_statistics::CullingIndirectRenderViewStatistics;
use amber_lume::render::pass::pass_statistics::PassStatistics;
use amber_lume::resources::index::index_manager_statistics::IndexManagerStatistics;
use amber_lume::resources::range_allocator::range_allocator_statistics::RangeAllocatorStatistics;
use amber_lume::settings::settings::SwitchSetting;
use amber_lume::settings::settings_handler::EngineSettingsHandler;
use amber_lume::statistics::amber_lume_statistics::AmberLumeStatistics;
use amber_lume::ui::theme::Theme;
use amber_lume::ui::ui_state::UiFragmentState;
use crate::ui::widgets::tabs::tabs;

pub struct DebugFragmentState;

impl DebugFragmentState {
    pub fn create() -> Self {
        Self {}
    }
}

impl UiFragmentState for DebugFragmentState {
    fn render(&mut self, theme: &Theme, settings_handler: &EngineSettingsHandler, statistics: &AmberLumeStatistics) {
        tabs(&theme, &[
            ("Resource", &|| {
                pad(Pad::all(12.0), || {
                    let passes = &statistics.resources;
                    column(|| {
                        resource_usage_statistics("Pipeline", &passes.pipeline_provider.index);
                        resource_usage_statistics("Compute pipeline", &passes.compute_pipeline_provider.index);

                        resource_usage_statistics("Mesh", &passes.mesh_provider.index);
                        range_allocator_statistics("Indices", &passes.mesh_provider.backend.index);
                        range_allocator_statistics("Vertices", &passes.mesh_provider.backend.vertex);
                        range_allocator_statistics("Submeshes", &passes.mesh_provider.backend.submesh);
                        resource_usage_statistics("Skeleton", &passes.skeleton_provider.index);
                        range_allocator_statistics("Bones", &passes.skeleton_provider.backend.bone);
                        resource_usage_statistics("Material", &passes.material_provider.index);
                        resource_usage_statistics("Image", &passes.image_provider.index);
                    });
                });
            }),
            ("CPU", &|| {
                pad(Pad::all(12.0), || {
                    column(|| {
                        statistic_clipped_time("Total frame time", statistics.render.total_time);
                        statistic_clipped_time("Collect record commands", statistics.render.collect_record_commands);
                    });
                });
            }),
            ("Pass", &|| {
                pad(Pad::all(12.0), || {
                    column(|| {
                        pass_statistics("Culling", &statistics.render.passes_statistics.culling);
                        let view_count = statistics.render.passes_statistics.culling_meta.render_views.len();
                        let dispatch_time = statistics.render.passes_statistics.culling_meta.dispatch_time;

                        statistic_clipped_time("Culling dispatch", dispatch_time);

                        for i in 0..view_count {
                            let render_view = &statistics.render.passes_statistics.culling_meta.render_views[i];

                            render_view_statistics("Render view", &render_view);
                        }

                        pass_statistics("Depth", &statistics.render.passes_statistics.depth);

                        pass_statistics("Shadows", &statistics.render.passes_statistics.shadows);

                        pass_statistics("Shadow mask", &statistics.render.passes_statistics.shadow_mask);

                        pass_statistics("Physics debug", &statistics.render.passes_statistics.physics_debug);

                        pass_statistics("Main", &statistics.render.passes_statistics.main);

                        pass_statistics("UI", &statistics.render.passes_statistics.ui);
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
                        switch_option(settings_handler.get_pending().debug.physics_interpolation, |new_value| {
                            settings_handler.update(|settings| {
                                settings.debug.physics_interpolation.set(new_value);
                            })
                        });

                        let apply_button = button("Apply");
                        if apply_button.clicked {
                            settings_handler.apply();
                        }

                        let reset_button = button("Reset");
                        if reset_button.clicked {
                            settings_handler.reset();
                        }
                    });
                });
            }),
        ], 0);
    }
}

fn resource_usage_statistics(title: &str, value: &IndexManagerStatistics) {
    let capacity = value.capacity;
    let used = value.used;
    let grave = value.grave;

    let mut text = Text::new(16.0, format!("{} indices: {}/{}, grave {}", title, used, capacity, grave));
    text.style.color = Color::WHITE;
    text.show();
}

fn range_allocator_statistics(title: &str, value: &RangeAllocatorStatistics) {
    let mut text = Text::new(16.0, format!(
        "{} used {}/{}, largest {}, fragmentation: {}",
        title,
        value.used,
        value.capacity,
        value.largest_free,
        value.fragmentation,
    ));
    text.style.color = Color::WHITE;
    text.show();
}

fn statistic_clipped_time(title: &str, value: u64) {
    let value = value as f32 / 1_000_000.0;

    let mut text = Text::new(16.0, format!("{}: {:.3}ms", title, value));
    text.style.color = Color::WHITE;
    text.show();
}

fn pass_statistics(title: &str, value: &PassStatistics) {
    let prepare = value.prepare as f32 / 1_000_000.0;
    let collect_render_commands = value.collect_render_commands as f32 / 1_000_000.0;

    let mut text = Text::new(16.0, format!("Pass {}: prepare {:.3}ms, collect commands {:.3}ms", title, prepare, collect_render_commands));
    text.style.color = Color::WHITE;
    text.show();
}

fn render_view_statistics(
    title: &str,
    render_view: &CullingIndirectRenderViewStatistics,
) {
    let mut text = Text::new(16.0, format!(
        "{}: rendered {}, culled {}",
        title,
        render_view.submeshes_rendered,
        render_view.submeshes_culled,
    ));
    text.style.color = Color::WHITE;
    text.show();
}

// fn usage_statistic(title: &str, usage: &Option<IndicesUsageStatistics>) {
//     let value = if let Some(usage) = usage {
//         let usage_percentage = (usage.used as f32 / usage.capacity as f32)  * 100.0;
//
//         format!("{}/{} ({:.2}%) grave: {}", usage.used, usage.capacity, usage_percentage, usage.grave)
//     } else {
//         String::from("none")
//     };
//
//     pad(Pad::all(4.0), || {
//         let mut text = Text::new(16.0, format!("{}: {}", title, value));
//         text.style.color = Color::WHITE;
//         text.show();
//     });
// }

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
