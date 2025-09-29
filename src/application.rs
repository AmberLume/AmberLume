use std::sync::Arc;
use tracing::trace;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes, WindowId};

pub struct Application {
    attributes: WindowAttributes,

    window: Option<Arc<Window>>,
    window_id: Option<WindowId>,
}

impl Application {
    pub fn new(attributes: WindowAttributes) -> Self {
        trace!("Creating Application...");
        
        Self {
            attributes,

            window: None,
            window_id: None,
        }
    }
}

impl ApplicationHandler for Application {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        trace!("Resuming Application...");
        
        let window = Arc::new(event_loop.create_window(self.attributes.clone()).unwrap());
        self.window = Some(window.clone());
        self.window_id = Some(window.id());
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        if Some(window_id) != self.window_id { return; }

        trace!("Received window event: {:?}", event);
        
        match event {
            WindowEvent::Resized(_) => {

            }
            WindowEvent::RedrawRequested => {
                
            }
            WindowEvent::CloseRequested => {
                
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
