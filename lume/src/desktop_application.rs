use crate::lume::Lume;
use std::sync::Arc;
use tracing::{error, info, instrument, trace, warn};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes, WindowId};

pub struct Application {
    attributes: WindowAttributes,

    window: Option<Arc<Window>>,

    lume: Option<Lume>,
}

impl Application {
    pub fn new(attributes: WindowAttributes) -> Self {
        trace!("Creating Application...");

        Self {
            attributes,

            window: None,

            lume: None,
        }
    }
}

impl ApplicationHandler for Application {
    #[instrument(level = "trace", skip_all)]
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window = Arc::new(event_loop.create_window(self.attributes.clone()).unwrap());

            trace!("Window created");

            match Lume::create(window.clone()) {
                Ok(lume) => {
                    self.window = Some(window.clone());

                    self.lume = Some(lume);
                }
                Err(e) => {
                    error!("Failed to create Lume: {}", e);
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
                    if let Some(lume) = self.lume.as_mut() {
                        match lume.on_update_surface() {
                            Ok(_) => info!("Window resized successfully"),
                            Err(error) => warn!("Window resized failed: {}", error),
                        }
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(lume) = self.lume.as_mut() {
                    if let Err(e) = lume.draw() {
                        error!("Failed to draw frame: {:?}", e);

                        event_loop.exit();
                    }
                }
            }
            WindowEvent::CloseRequested => {
                info!("Close requested");

                if let Some(lume) = self.lume.as_mut() {
                    match lume.on_close() {
                        Ok(_) => info!("Window closed successfully"),
                        Err(error) => warn!("Window closed with error: {}", error),
                    }
                }

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
