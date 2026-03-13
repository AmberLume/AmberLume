use std::sync::Arc;
use yakui::{Rect, Vec2, Yakui};
use anyhow::Result;
use ash::vk::Extent2D;
use yakui::event::Event;
use yakui::input::MouseButton as YakuiMouseButton;
use crate::ids::FrameIndex;
use crate::render::vulkan::buffer::buffer_manager::BufferManager;
use crate::render::vulkan::render_pass::ui_render_pass::ui_snapshot::UiSnapshot;
use crate::render::vulkan::resource_loader::ResourceLoader;
use crate::resources::descriptor_index_managers::IndexManagers;
use crate::resources::persistent::persistent_resources::PersistentResources;
use crate::resources::resource_factories::ResourceFactories;
use crate::ui::events::ui_events::{EventState, MouseButton, MouseEvent};
use crate::ui::ui_renderer::UiRenderer;
use crate::ui::ui_resource_manager::UiResourceManager;

pub struct UiContext {
    handle: Yakui,

    ui_resource_manager: UiResourceManager,
    ui_renderer: Arc<dyn UiRenderer>,

    pub window_size: Vec2,
}

impl UiContext {
    pub fn new(
        resource_factories: Arc<ResourceFactories>,
        index_managers: Arc<IndexManagers>,
        persistent_resources: Arc<PersistentResources>,
        buffer_manager: Arc<BufferManager>,
        resource_loader: Arc<ResourceLoader>,
        ui_renderer: Arc<dyn UiRenderer>,
    ) -> Result<Self> {
        let handle = Yakui::new();

        let ui_resource_manager = UiResourceManager::new(
            resource_factories,
            index_managers,
            persistent_resources,
            buffer_manager,
            resource_loader,
        );

        Ok(Self {
            handle,

            ui_resource_manager,
            ui_renderer,

            window_size: Vec2::ZERO,
        })
    }

    pub fn render_ui(
        &mut self,
        extent: Extent2D,
    ) {
        let size = Vec2::new(extent.width as f32, extent.height as f32);

        self.window_size = size;

        self.handle.set_surface_size(size);
        self.handle.set_unscaled_viewport(Rect::from_pos_size(Vec2::ZERO, size));

        self.handle.start();

        self.ui_renderer.render(&self);

        self.handle.finish();
    }

    pub fn build_ui_snapshot(&mut self, frame_index: FrameIndex) -> Result<UiSnapshot> {
        let paint_dom = self.handle.paint();

        self.ui_resource_manager.collect_resource_edits(&paint_dom)?;

        self.ui_resource_manager.fill_draw_buffers(&paint_dom, frame_index)
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
