use std::sync::Arc;
use std::time::Instant;
use yakui::{Color, Rect, Vec2, Yakui};
use anyhow::Result;
use ash::vk::Extent2D;
use yakui::event::Event;
use yakui::input::MouseButton as YakuiMouseButton;
use yakui::paint::Vertex;
use crate::limits::renderer_limits::RendererLimits;
use crate::render::factories::buffer::builder::buffer_info::BufferInfo;
use crate::ui::buffer::ui_index_buffer::create_ui_index_buffer;
use crate::ui::buffer::ui_vertex_buffer::create_ui_vertex_buffer;
use crate::render::factories::buffer::frame_buffer::frame_buffer::FrameBuffer;
use crate::render::factories::buffer::managed_buffer_factory::ManagedBufferFactory;
use crate::render::factories::buffer::slice_buffer::slice_buffer::SliceBuffer;
use crate::render::pass::ui::ui_snapshot::UiSnapshot;
use crate::resources::dynamic::image::image_backend::ImageBackend;
use crate::resources::dynamic::resource_provider::ResourceProvider;
use crate::resources::persistent::persistent_resources::PersistentResources;
use crate::settings::settings_handler::EngineSettingsHandler;
use crate::statistics::amber_lume_statistics::AmberLumeStatistics;
use crate::ui::events::ui_events::{EventState, MouseButton, MouseEvent};
use crate::ui::theme::Theme;
use crate::ui::ui_renderer::UiRenderer;
use crate::ui::ui_resource_manager::UiResourceManager;

pub struct UiContext {
    handle: Yakui,

    ui_resource_manager: UiResourceManager,
    ui_renderer: Arc<dyn UiRenderer>,

    pub index_buffer: FrameBuffer<SliceBuffer<u32>>,
    pub vertex_buffer: FrameBuffer<SliceBuffer<Vertex>>,

    time: Instant,

    pub theme: Theme,
    
    pub delta_time: f32,
}

impl UiContext {
    pub fn new(
        renderer_limits: &RendererLimits,
        buffer_factory: &ManagedBufferFactory,
        image_provider: Arc<ResourceProvider<ImageBackend>>,
        persistent_resources: Arc<PersistentResources>,
        ui_renderer: Arc<dyn UiRenderer>,
    ) -> Result<Self> {
        let handle = Yakui::new();

        let ui_resource_manager = UiResourceManager::new(
            persistent_resources,
            image_provider,
        );

        let index_buffer = create_ui_index_buffer(
            &buffer_factory,
            renderer_limits.frames_in_flight,
            100000,
        )?;
        let vertex_buffer = create_ui_vertex_buffer(
            &buffer_factory,
            renderer_limits.frames_in_flight,
            100000,
        )?;

        Ok(Self {
            handle,

            ui_resource_manager,
            ui_renderer,

            index_buffer,
            vertex_buffer,

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
        statistics: &AmberLumeStatistics,
    ) {
        let size = Vec2::new(extent.width as f32, extent.height as f32);

        let now = Instant::now();
        let delta_time = now - self.time;

        self.time = now;
        self.delta_time = delta_time.as_secs_f32();

        self.handle.set_surface_size(size);
        self.handle.set_unscaled_viewport(Rect::from_pos_size(Vec2::ZERO, size));

        self.handle.start();

        self.ui_renderer.render(&self, settings_handler, statistics);

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

    pub fn destroy(self, buffer_factory: &ManagedBufferFactory) -> Result<()> {
        self.ui_resource_manager.destroy();

        buffer_factory.destroy_buffer(self.vertex_buffer.into_managed_buffer())?;
        buffer_factory.destroy_buffer(self.index_buffer.into_managed_buffer())?;

        Ok(())
    }
}
