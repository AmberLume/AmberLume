use std::sync::Arc;
use std::time::Instant;
use yakui::{Color, Rect, Vec2, Yakui};
use anyhow::Result;
use ash::vk::Extent2D;
use yakui::event::Event;
use yakui::input::MouseButton as YakuiMouseButton;
use crate::render::render_pass::ui::ui_snapshot::UiSnapshot;
use crate::render::resources::resource_loader::ResourceLoader;
use crate::resources::descriptor_index_managers::IndexManagers;
use crate::resources::persistent::persistent_resources::PersistentResources;
use crate::resources::resource_factories::ResourceFactories;
use crate::settings::settings_handler::EngineSettingsHandler;
use crate::ui::events::ui_events::{EventState, MouseButton, MouseEvent};
use crate::ui::theme::Theme;
use crate::ui::ui_renderer::UiRenderer;
use crate::ui::ui_resource_manager::UiResourceManager;

pub struct UiContext {
    handle: Yakui,

    ui_resource_manager: UiResourceManager,
    ui_renderer: Arc<dyn UiRenderer>,

    time: Instant,

    pub theme: Theme,
    
    pub delta_time: f32,
}

impl UiContext {
    pub fn new(
        resource_factories: Arc<ResourceFactories>,
        index_managers: Arc<IndexManagers>,
        persistent_resources: Arc<PersistentResources>,
        resource_loader: Arc<ResourceLoader>,
        ui_renderer: Arc<dyn UiRenderer>,
    ) -> Result<Self> {
        let handle = Yakui::new();

        let ui_resource_manager = UiResourceManager::new(
            resource_factories,
            index_managers,
            persistent_resources,
            resource_loader,
        );

        Ok(Self {
            handle,

            ui_resource_manager,
            ui_renderer,

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
        settings_handler: &EngineSettingsHandler,
    ) {
        let size = Vec2::new(extent.width as f32, extent.height as f32);

        let now = Instant::now();
        let delta_time = now - self.time;

        self.time = now;
        self.delta_time = delta_time.as_secs_f32();

        self.handle.set_surface_size(size);
        self.handle.set_unscaled_viewport(Rect::from_pos_size(Vec2::ZERO, size));

        self.handle.start();

        self.ui_renderer.render(&self, settings_handler);

        self.handle.finish();
    }

    pub fn build_ui_snapshot(&mut self) -> Result<UiSnapshot> {
        let paint_dom = self.handle.paint();

        self.ui_resource_manager.collect_resource_edits(&paint_dom)?;

        self.ui_resource_manager.fill_draw_buffers(&paint_dom)
    }

    pub fn on_mouse_event(&mut self, event: MouseEvent) {
        let event = match event {
            MouseEvent::Move { position } => Event::CursorMoved(Some(Vec2::from_array(position))),
            MouseEvent::Click { button, state } => {
                Event::MouseButtonChanged {
                    button: match button {
                        MouseButton::Left => YakuiMouseButton::One,
                        MouseButton::Right => YakuiMouseButton::Two,
                        MouseButton::Middle => YakuiMouseButton::Three,
                        MouseButton::Forward => { return; }
                        MouseButton::Back => { return; }
                    },
                    down: state == EventState::Down,
                }
            }
            MouseEvent::Scroll { delta } => {
                Event::MouseScroll {
                    delta: Vec2::from_array(delta),
                }
            }
        };

        self.handle.handle_event(event);
    }

    pub fn destroy(self) -> Result<()> {
        self.ui_resource_manager.destroy()?;

        Ok(())
    }
}
