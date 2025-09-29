use std::sync::Arc;
use tracing::{error, info, instrument, trace};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes, WindowId};
use crate::vulkan::vk_app::VkApp;

pub struct Application {
    attributes: WindowAttributes,

    window: Option<Arc<Window>>,

    vk_app: Option<VkApp>
}

impl Application {
    pub fn new(attributes: WindowAttributes) -> Self {
        trace!("Creating Application...");
        
        Self {
            attributes,

            window: None,
            
            vk_app: None,
        }
    }
}

impl ApplicationHandler for Application {
    #[instrument(level = "trace", skip_all)]
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window = Arc::new(event_loop.create_window(self.attributes.clone()).unwrap());

            trace!("Window created");

            let vk_app = VkApp::new(&window);

            match vk_app {
                Ok(vk_app) => {
                    self.vk_app = Some(vk_app);
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
    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        let Some(window) = self.window.as_ref() else { return; };
        if window.id() != window_id { return; };
        
        match event {
            WindowEvent::Resized(size) => {
                if size.width > 0 && size.height > 0 {
                    if let Some(vk_app) = self.vk_app.as_mut() {
                        if let Err(e) = vk_app.recreate_swapchain(window) {
                            error!("Failed to recreate swapchain: {:?}", e);
                            
                            event_loop.exit();
                        }
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(vk_app) = self.vk_app.as_mut() {
                    if let Err(e) = vk_app.draw_frame(window) {
                        error!("Failed to draw frame: {:?}", e);
                        
                        event_loop.exit();
                    }
                }
            }
            WindowEvent::CloseRequested => {
                info!("Close requested");

                event_loop.exit();
            }
            _ => {

            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        trace!("About to wait called");
        
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}
