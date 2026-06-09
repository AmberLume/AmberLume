use yakui::{button, checkbox, column, pad, text, Color, Constraints, CrossAxisAlignment, MainAxisAlignment, Vec2};
use yakui::widgets::{ConstrainedBox, List, Pad, Slider, Text};
use amber_lume::input_handler::input_frame::InputFrame;
use amber_lume::profiler::frame_profile::{CpuMetaEntry, FrameProfile, ZoneEntry};
use amber_lume::profiler::meta_value::MetaValue;
use amber_lume::profiler::zone::ZoneKind;
use amber_lume::render::pass::culling_indirect::render_view_culling_indirect_statistics::{CASCADE_CULLING_META_NAME, CullingIndirectRenderViewStatisticsGPU, MAIN_CULLING_META_NAME};
use amber_lume::render::pass::sdsm::cascade_statistics::{CASCADE_COMPUTE_META_NAME, CascadeStatisticsGPU};
use amber_lume::resources::index::index_manager_statistics::IndexManagerStatistics;
use amber_lume::resources::range_allocator::range_allocator_statistics::RangeAllocatorStatistics;
use amber_lume::editor::editor_state::EditorState;
use amber_lume::settings::settings::{RangeSetting, SwitchSetting};
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
    fn render(
        &mut self,
        theme: &Theme,
        _input_frame: &InputFrame,
        settings_handler: &EngineSettingsHandler,
        statistics: &AmberLumeStatistics,
        _editor_state: &EditorState,
    ) {
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
                        resource_usage_statistics("Animation", &passes.animation_provider.index);
                        range_allocator_statistics("Animation frame", &passes.animation_provider.backend.frames);
                        resource_usage_statistics("Material", &passes.material_provider.index);
                        resource_usage_statistics("Image", &passes.image_provider.index);

                        range_statistics("UI index", statistics.ui.index_capacity, statistics.ui.index_used);
                        range_statistics("UI vertex", statistics.ui.vertex_capacity, statistics.ui.vertex_used);
                    });
                });
            }),
            ("CPU", &|| {
                pad(Pad::all(12.0), || {
                    column(|| {
                        for zone in statistics.frame_profile.zones.iter().filter(|z| z.kind == ZoneKind::Cpu) {
                            let depth = zone_depth(&statistics.frame_profile.zones, zone);
                            let indent: String = "  ".repeat(depth);

                            statistic_clipped_time(&format!("{}{}", indent, zone.name), zone.duration_ns);
                        }
                    });
                });
            }),
            ("GPU", &|| {
                pad(Pad::all(12.0), || {
                    column(|| {
                        for zone in statistics.frame_profile.zones.iter().filter(|z| z.kind == ZoneKind::Gpu) {
                            statistic_clipped_time(zone.name, zone.duration_ns);
                        }
                    });
                });
            }),
            ("Meta", &|| {
                pad(Pad::all(12.0), || {
                    column(|| {
                        for entry in &statistics.frame_profile.cpu_meta {
                            cpu_meta_entry(entry);
                        }

                        culling_meta("Main culling", &statistics.frame_profile, MAIN_CULLING_META_NAME);
                        culling_meta("Cascade culling", &statistics.frame_profile, CASCADE_CULLING_META_NAME);
                        cascade_compute_meta(&statistics.frame_profile);
                    });
                });
            }),
            ("Heap", &|| {
                pad(Pad::all(12.0), || {
                    column(|| {
                        heap_statistics(
                            "CpuToGpu buffer",
                            statistics.render.cpu_to_gpu_allocator_statistics.capacity,
                            statistics.render.cpu_to_gpu_allocator_statistics.used,
                        );
                    });
                });
            }),
            ("Render", &|| {
                pad(Pad::all(12.0), || {
                    column(|| {
                        slider_option(settings_handler.get_pending().render.exposure, |new_value| {
                            settings_handler.update(|settings| {
                                settings.render.exposure.set(new_value);
                            });
                            settings_handler.apply();
                        });
                        slider_option(settings_handler.get_pending().render.bloom_intensity, |new_value| {
                            settings_handler.update(|settings| {
                                settings.render.bloom_intensity.set(new_value);
                            });
                            settings_handler.apply();
                        });
                        slider_option(settings_handler.get_pending().render.bloom_threshold, |new_value| {
                            settings_handler.update(|settings| {
                                settings.render.bloom_threshold.set(new_value);
                            });
                            settings_handler.apply();
                        });

                        if statistics.render.hdr_supported {
                            switch_option(settings_handler.get_pending().render.hdr, |new_value| {
                                settings_handler.update(|settings| {
                                    settings.render.hdr.set(new_value);
                                });
                                settings_handler.apply();
                            });
                            slider_option(settings_handler.get_pending().render.paper_white, |new_value| {
                                settings_handler.update(|settings| {
                                    settings.render.paper_white.set(new_value);
                                });
                                settings_handler.apply();
                            });
                        } else {
                            let mut text = Text::new(16.0, String::from("HDR: not supported"));
                            text.style.color = Color::rgb(128, 128, 128);
                            text.show();
                        }
                    });
                });
            }),
            ("Light", &|| {
                pad(Pad::all(12.0), || {
                    column(|| {
                        switch_option(settings_handler.get_pending().light.auto_day_night, |new_value| {
                            settings_handler.update(|settings| {
                                settings.light.auto_day_night.set(new_value);
                            });
                            settings_handler.apply();
                        });
                        slider_option(settings_handler.get_pending().light.direction_x, |new_value| {
                            settings_handler.update(|settings| {
                                settings.light.direction_x.set(new_value);
                            });
                            settings_handler.apply();
                        });
                        slider_option(settings_handler.get_pending().light.direction_y, |new_value| {
                            settings_handler.update(|settings| {
                                settings.light.direction_y.set(new_value);
                            });
                            settings_handler.apply();
                        });
                        slider_option(settings_handler.get_pending().light.direction_z, |new_value| {
                            settings_handler.update(|settings| {
                                settings.light.direction_z.set(new_value);
                            });
                            settings_handler.apply();
                        });
                        slider_option(settings_handler.get_pending().light.color_r, |new_value| {
                            settings_handler.update(|settings| {
                                settings.light.color_r.set(new_value);
                            });
                            settings_handler.apply();
                        });
                        slider_option(settings_handler.get_pending().light.color_g, |new_value| {
                            settings_handler.update(|settings| {
                                settings.light.color_g.set(new_value);
                            });
                            settings_handler.apply();
                        });
                        slider_option(settings_handler.get_pending().light.color_b, |new_value| {
                            settings_handler.update(|settings| {
                                settings.light.color_b.set(new_value);
                            });
                            settings_handler.apply();
                        });
                        slider_option(settings_handler.get_pending().light.intensity, |new_value| {
                            settings_handler.update(|settings| {
                                settings.light.intensity.set(new_value);
                            });
                            settings_handler.apply();
                        });
                        slider_option(settings_handler.get_pending().light.ambient, |new_value| {
                            settings_handler.update(|settings| {
                                settings.light.ambient.set(new_value);
                            });
                            settings_handler.apply();
                        });
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
                        switch_option(settings_handler.get_pending().debug.physics_paused, |new_value| {
                            settings_handler.update(|settings| {
                                settings.debug.physics_paused.set(new_value);
                            });
                            settings_handler.apply();
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

fn range_statistics(title: &str, capacity: u32, used: u32) {
    let mut text = Text::new(16.0, format!(
        "{} used {}/{}",
        title,
        used,
        capacity,
    ));
    text.style.color = Color::WHITE;
    text.show();
}

fn heap_statistics(title: &str, capacity: u32, used: u32) {
    let percentage = used as f32 / capacity as f32 * 100.0;

    let mut text = Text::new(16.0, format!(
        "{} used {}/{} ({:.3}%)",
        title,
        used,
        capacity,
        percentage,
    ));
    text.style.color = Color::WHITE;
    text.show();
}

fn resource_usage_statistics(title: &str, value: &IndexManagerStatistics) {
    let capacity = value.capacity;
    let used = value.used;
    let grave = value.grave;

    let percentage = used as f32 / capacity as f32 * 100.0;

    let mut text = Text::new(16.0, format!("{} indices: {}/{} ({:.3}%), grave {}", title, used, capacity, percentage, grave));
    text.style.color = Color::WHITE;
    text.show();
}

fn range_allocator_statistics(title: &str, value: &RangeAllocatorStatistics) {
    let percentage = value.used as f32 / value.capacity as f32 * 100.0;
    
    let mut text = Text::new(16.0, format!(
        "{} used {}/{} ({:.3}%), largest {}, fragmentation: {}",
        title,
        value.used,
        value.capacity,
        percentage,
        value.largest_free,
        value.fragmentation,
    ));
    text.style.color = Color::WHITE;
    text.show();
}

fn statistic_clipped_time(title: &str, value: u64) {
    let mut text = Text::new(16.0, format!(
        "{}: {:.3}ms",
        title,
        from_ns_to_ms(value),
    ));
    text.style.color = Color::WHITE;
    text.show();
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

fn slider_option(setting: RangeSetting, on_change: impl FnOnce(f32)) {
    let value = format!("{}: {:.2}", setting.get_title(), setting.get());

    let mut row = List::row();
    row.main_axis_alignment = MainAxisAlignment::Start;
    row.cross_axis_alignment = CrossAxisAlignment::Center;

    row.show(|| {
        ConstrainedBox::new(Constraints {
            min: Vec2::new(140.0, 0.0),
            max: Vec2::new(140.0, f32::INFINITY),
        }).show(|| {
            text(16.0, value);
        });

        pad(Pad::all(4.0), || {
            ConstrainedBox::new(Constraints {
                min: Vec2::new(450.0, 0.0),
                max: Vec2::new(450.0, f32::INFINITY),
            }).show(|| {
                let slider = Slider::new(
                    setting.get() as f64,
                    setting.get_min() as f64,
                    setting.get_max() as f64,
                ).show();

                if let Some(new_value) = slider.value {
                    on_change(new_value as f32);
                }
            });
        });
    });
}

fn from_ns_to_ms(ns: u64) -> f32 {
    ns as f32 / 1_000_000.0
}

fn culling_meta(title: &str, frame_profile: &FrameProfile, name: &str) {
    let Some(views) = frame_profile.gpu_meta_for::<Vec<CullingIndirectRenderViewStatisticsGPU>>(name) else {
        return;
    };

    let mut header = Text::new(16.0, format!("{}:", title));
    header.style.color = Color::WHITE;
    header.show();

    for (index, view) in views.iter().enumerate() {
        if view.submeshes_rendered == 0 && view.submeshes_culled == 0 {
            continue;
        }

        let mut text = Text::new(16.0, format!(
            "  view {}: rendered {}, culled {}",
            index,
            view.submeshes_rendered,
            view.submeshes_culled,
        ));
        text.style.color = Color::WHITE;
        text.show();
    }
}

fn cpu_meta_entry(entry: &CpuMetaEntry) {
    let value = match entry.value {
        MetaValue::U32(v) => format!("{}", v),
        MetaValue::U64(v) => format!("{}", v),
        MetaValue::F32(v) => format!("{:.3}", v),
        MetaValue::F64(v) => format!("{:.3}", v),
    };

    let mut text = Text::new(16.0, format!("{}: {}", entry.name, value));
    text.style.color = Color::WHITE;
    text.show();
}

fn cascade_compute_meta(frame_profile: &FrameProfile) {
    let Some(cascades) = frame_profile.gpu_meta_for::<Vec<CascadeStatisticsGPU>>(CASCADE_COMPUTE_META_NAME) else {
        return;
    };

    let z_max = cascades.first().map(|c| c.z_max).unwrap_or(0.0);

    let mut header = Text::new(16.0, format!("SDSM: max distance {:.2}m", z_max));
    header.style.color = Color::WHITE;
    header.show();

    let mut cascades_header = Text::new(16.0, String::from("Cascades:"));
    cascades_header.style.color = Color::WHITE;
    cascades_header.show();

    for (index, cascade) in cascades.iter().enumerate() {
        let mut text = Text::new(16.0, format!(
            "  {}: {:.2}m -> {:.2}m, radius {:.2}m",
            index,
            cascade.range_start,
            cascade.range_end,
            cascade.world_radius,
        ));
        text.style.color = Color::WHITE;
        text.show();
    }
}

fn zone_depth(zones: &[ZoneEntry], zone: &ZoneEntry) -> usize {
    let mut depth = 0;
    let mut current = zone.parent;
    while let Some(parent_id) = current {
        depth += 1;
        current = zones.iter().find(|z| z.id == parent_id).and_then(|z| z.parent);
    }
    depth
}
