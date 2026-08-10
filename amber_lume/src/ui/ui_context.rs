use std::sync::Arc;
use std::time::Instant;
use yakui::{Color, Rect, Vec2, Yakui};
use anyhow::Result;
use ash::vk::Extent2D;
use yakui::event::Event;
use yakui::input::MouseButton as YakuiMouseButton;
use crate::editor::editor_state::EditorState;
use input::{HardwarePointerKeyCodes, InputHandler};
use crate::render::pass::ui::ui_frame::UiFrame;
use resource_store::PersistentResources;
use resource_residency::ResourceProvider;
use resource_store::ImageBackend;
use settings::EngineSettingsHandler;
use crate::statistics::amber_lume_statistics::AmberLumeStatistics;
use crate::ui::theme::Theme;
use crate::ui::ui_renderer::UiRenderer;
use crate::ui::ui_resource_manager::UiResourceManager;
use crate::ui::ui_statistics::UiStatistics;

pub struct UiContext {
    handle: Yakui,

    ui_resource_manager: UiResourceManager,
    ui_renderer: Arc<dyn UiRenderer>,

    index_used: u32,
    vertex_used: u32,

    time: Instant,

    pub theme: Theme,

    pub delta_time: f32,
}

impl UiContext {
    pub fn new(
        image_provider: Arc<ResourceProvider<ImageBackend>>,
        persistent_resources: Arc<PersistentResources>,
        ui_renderer: Arc<dyn UiRenderer>,
    ) -> Result<Self> {
        let handle = Yakui::new();

        let ui_resource_manager = UiResourceManager::new(
            persistent_resources,
            image_provider,
        );

        Ok(Self {
            handle,

            ui_resource_manager,
            ui_renderer,

            index_used: 0,
            vertex_used: 0,

            time: Instant::now(),

            theme: Theme {
                background: Color::BLACK,
                surface: Color::rgba(50, 50, 50, 255),
            },

            delta_time: 0.0,
        })
    }

    pub fn render_ui(
        &mut self,
        extent: Extent2D,
        input: &mut InputHandler,
        settings_handler: &EngineSettingsHandler,
        statistics: &AmberLumeStatistics,
        editor_state: &EditorState,
    ) {
        let size = Vec2::new(extent.width as f32, extent.height as f32);

        let now = Instant::now();
        let delta_time = now - self.time;

        self.time = now;
        self.delta_time = delta_time.as_secs_f32();

        self.handle.set_surface_size(size);
        self.handle.set_unscaled_viewport(Rect::from_pos_size(Vec2::ZERO, size));

        self.handle.start();

        self.ui_renderer.render(&self, input, settings_handler, statistics, editor_state);

        self.handle.finish();
    }

    pub fn build_ui_frame(&mut self) -> Result<UiFrame> {
        let paint_dom = self.handle.paint();

        self.ui_resource_manager.collect_resource_edits(&paint_dom)?;

        let result = self.ui_resource_manager.fill_draw_buffers(&paint_dom)?;
        
        self.index_used = result.indices.len() as u32;
        self.vertex_used = result.vertices.len() as u32;
        
        Ok(result)
    }

    pub fn handle_input(&mut self, input: &mut InputHandler) {
        if let Some(position) = input.position(false) {
            let sunk = self.handle.handle_event(Event::CursorMoved(Some(Vec2::new(position.x, position.y))));

            if sunk {
                input.position(true);
            }
        }

        if let Some(scroll) = input.scroll(false) {
            let sunk = self.handle.handle_event(Event::MouseScroll {
                delta: Vec2::new(scroll.x, scroll.y),
            });

            if sunk {
                input.scroll(true);
            }
        }

        for index in 0..HardwarePointerKeyCodes::Count as u32 {
            let code = HardwarePointerKeyCodes::from_index(index);

            let button = match code {
                HardwarePointerKeyCodes::Left => YakuiMouseButton::One,
                HardwarePointerKeyCodes::Right => YakuiMouseButton::Two,
                HardwarePointerKeyCodes::Middle => YakuiMouseButton::Three,
                _ => continue,
            };

            if input.button(code, false).is_just_pressed() {
                let sunk = self.handle.handle_event(Event::MouseButtonChanged { button, down: true });

                if sunk {
                    input.button(code, true);
                }
            }

            if input.button(code, false).is_just_released() {
                let sunk = self.handle.handle_event(Event::MouseButtonChanged { button, down: false });

                if sunk {
                    input.button(code, true);
                }
            }
        }
    }
    
    pub fn statistics(&self) -> UiStatistics {
        UiStatistics {
            index_used: self.index_used,
            vertex_used: self.vertex_used,
        }
    }

    pub fn destroy(self) -> Result<()> {
        self.ui_resource_manager.destroy();

        Ok(())
    }
}
