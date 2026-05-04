use std::sync::Arc;
use std::time::Instant;
use yakui::{Color, Rect, Vec2, Yakui};
use anyhow::Result;
use ash::vk::Extent2D;
use yakui::event::Event;
use yakui::input::MouseButton as YakuiMouseButton;
use yakui::paint::Vertex;
use crate::input_handler::hardware_pointer_key_codes::HardwarePointerKeyCodes;
use crate::input_handler::hardware_key_state::HardwareKeyState;
use crate::input_handler::input_frame::InputFrame;
use crate::render::factories::buffer::builder::buffer_info::BufferInfo;
use crate::ui::buffer::ui_index_buffer::create_ui_index_buffer;
use crate::ui::buffer::ui_vertex_buffer::create_ui_vertex_buffer;
use crate::render::factories::buffer::frame_buffer::frame_buffer::FrameBuffer;
use crate::render::factories::buffer::managed_buffer_factory::ManagedBufferFactory;
use crate::render::factories::buffer::slice_buffer::slice_buffer::SliceBuffer;
use crate::render::pass::ui::ui_snapshot::UiSnapshot;
use crate::resources::store::persistent::persistent_resources::PersistentResources;
use crate::resources::store::providers::resource_provider::ResourceProvider;
use crate::resources::store::providers::image::image_backend::ImageBackend;
use crate::settings::settings_handler::EngineSettingsHandler;
use crate::statistics::amber_lume_statistics::AmberLumeStatistics;
use crate::ui::theme::Theme;
use crate::ui::ui_renderer::UiRenderer;
use crate::ui::ui_resource_manager::UiResourceManager;
use crate::ui::ui_statistics::UiStatistics;

pub struct UiContext {
    handle: Yakui,

    ui_resource_manager: UiResourceManager,
    ui_renderer: Arc<dyn UiRenderer>,

    index_capacity: u32,
    index_used: u32,
    
    vertex_capacity: u32,
    vertex_used: u32,
    
    pub index_buffer: FrameBuffer<SliceBuffer<u32>>,
    pub vertex_buffer: FrameBuffer<SliceBuffer<Vertex>>,

    time: Instant,

    pub theme: Theme,
    
    pub delta_time: f32,
}

impl UiContext {
    pub fn new(
        frame_count: u32,
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
            frame_count,
            100000,
        )?;
        let vertex_buffer = create_ui_vertex_buffer(
            &buffer_factory,
            frame_count,
            100000,
        )?;

        Ok(Self {
            handle,

            ui_resource_manager,
            ui_renderer,

            index_capacity: 100000,
            index_used: 0,
            
            vertex_capacity: 100000,
            vertex_used: 0,
            
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

        let result = self.ui_resource_manager.fill_draw_buffers(&paint_dom)?;
        
        self.index_used = result.indices.len() as u32;
        self.vertex_used = result.vertices.len() as u32;
        
        Ok(result)
    }

    pub fn handle_input(&mut self, input_frame: &InputFrame) {
        for pointer in input_frame.get_pointer() {
            if pointer.position_delta != (0.0, 0.0) {
                let position = Vec2::new(pointer.position.0, pointer.position.1);

                self.handle.handle_event(Event::CursorMoved(Some(position)));
            }

            if pointer.scroll_delta != (0.0, 0.0) {
                let delta = Vec2::new(pointer.scroll_delta.0, pointer.scroll_delta.1);

                self.handle.handle_event(Event::MouseScroll { delta });
            }

            for (i, &button) in pointer.buttons.iter().enumerate() {
                if button == HardwareKeyState::JustPressed || button == HardwareKeyState::JustReleased {
                    self.handle.handle_event(Event::MouseButtonChanged {
                        button: match HardwarePointerKeyCodes::from_index(i as u32) {
                            HardwarePointerKeyCodes::Left => YakuiMouseButton::One,
                            HardwarePointerKeyCodes::Right => YakuiMouseButton::Two,
                            HardwarePointerKeyCodes::Middle => YakuiMouseButton::Three,
                            HardwarePointerKeyCodes::Forward => { continue; }
                            HardwarePointerKeyCodes::Backward => { continue; }
                            HardwarePointerKeyCodes::Count => { unreachable!(); }
                        },
                        down: button == HardwareKeyState::JustPressed,
                    });
                }
            }
        }
    }
    
    pub fn statistics(&self) -> UiStatistics {
        UiStatistics {
            index_capacity: self.index_capacity,
            index_used: self.index_used,
            
            vertex_capacity: self.vertex_capacity,
            vertex_used: self.vertex_used,
        }
    }

    pub fn destroy(self, buffer_factory: &ManagedBufferFactory) -> Result<()> {
        self.ui_resource_manager.destroy();

        buffer_factory.destroy_buffer(self.vertex_buffer.into_managed_buffer())?;
        buffer_factory.destroy_buffer(self.index_buffer.into_managed_buffer())?;

        Ok(())
    }
}
