use crate::render::vulkan::context_profile::ContextProfile;
use crate::render::vulkan::render_context::RenderContext;
use crate::render::vulkan::vk_context::VkContext;
use std::sync::Arc;
use tracing::{error, info, instrument, trace};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes, WindowId};

pub struct Application {
    attributes: WindowAttributes,

    window: Option<Arc<Window>>,

    vk_context: Option<Arc<VkContext>>,
    render_context: Option<RenderContext>,
}

impl Application {
    pub fn new(attributes: WindowAttributes) -> Self {
        trace!("Creating Application...");

        Self {
            attributes,

            window: None,

            vk_context: None,
            render_context: None,
        }
    }
}

impl ApplicationHandler for Application {
    #[instrument(level = "trace", skip_all)]
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window = Arc::new(event_loop.create_window(self.attributes.clone()).unwrap());

            trace!("Window created");

            let context_profile = ContextProfile::from(&window).unwrap();

            let vk_context = VkContext::new(context_profile);

            match vk_context {
                Ok(vk_context) => {
                    let vk_context = Arc::new(vk_context);

                    let render_context = RenderContext::create_from(
                        vk_context.clone(),
                        window.clone(),
                        [0.08, 0.10, 0.12, 1.0],
                    )
                    .unwrap();

                    self.vk_context = Some(vk_context);
                    self.render_context = Some(render_context);
                    self.window = Some(window);
                }
                Err(e) => {
                    error!("Failed to create VK app: {}", e);
                    event_loop.exit();
                }
            }
        }
    }

    #[instrument(level = "trace", skip_all)]
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(window) = self.window.as_ref() else {
            return;
        };
        if window.id() != window_id {
            return;
        };

        match event {
            WindowEvent::Resized(size) => {
                if size.width > 0 && size.height > 0 {
                    if let Some(render_context) = self.render_context.as_mut() {
                        if let Err(e) = render_context.recreate_swapchain() {
                            error!("Failed to recreate swapchain: {:?}", e);

                            event_loop.exit();
                        }
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(render_context) = self.render_context.as_mut() {
                    if let Err(e) = render_context.draw(window) {
                        error!("Failed to draw frame: {:?}", e);

                        event_loop.exit();
                    }
                }
            }
            WindowEvent::CloseRequested => {
                info!("Close requested");

                event_loop.exit();
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        trace!("About to wait called");

        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}
