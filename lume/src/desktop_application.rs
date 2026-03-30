use crate::lume::Lume;
use std::sync::Arc;
use tracing::{error, info, instrument, trace, warn};
use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton as WinitMouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowAttributes, WindowId};
use amber_lume::input_handler::input_event::KeyEvent;
use amber_lume::input_handler::keycodes::Keycode;
use anyhow::{bail, Result};
use amber_lume::ui::events::ui_events::{EventState, MouseButton, MouseEvent};

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

    fn key_to_amber_key(event: winit::event::KeyEvent) -> Result<KeyEvent> {
        let keycode = if let PhysicalKey::Code(code) = event.physical_key {
            match code {
                KeyCode::Escape => Keycode::Esc,
                KeyCode::ArrowUp => Keycode::ArrowUp,
                KeyCode::ArrowLeft => Keycode::ArrowLeft,
                KeyCode::ArrowDown => Keycode::ArrowDown,
                KeyCode::ArrowRight => Keycode::ArrowRight,
                KeyCode::KeyQ => Keycode::Q,
                KeyCode::KeyW => Keycode::W,
                KeyCode::KeyE => Keycode::E,
                KeyCode::KeyR => Keycode::R,
                KeyCode::KeyA => Keycode::A,
                KeyCode::KeyS => Keycode::S,
                KeyCode::KeyD => Keycode::D,
                KeyCode::KeyF => Keycode::F,
                KeyCode::KeyZ => Keycode::Z,
                KeyCode::KeyX => Keycode::X,
                KeyCode::KeyC => Keycode::C,
                KeyCode::KeyV => Keycode::V,
                _ => bail!("Received unhandled keycode: {:?}", event.physical_key),
            }
        } else {
            bail!("Received physical key is not a code: {:?}", event.physical_key);
        };

        Ok(match event.state {
            ElementState::Pressed => KeyEvent::Pressed(keycode),
            ElementState::Released => KeyEvent::Released(keycode),
        })
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
            WindowEvent::KeyboardInput { event, .. } => {
                if event.repeat {
                    return;
                };

                if let Some(lume) = self.lume.as_mut() {
                    if let Ok(key_event) = Self::key_to_amber_key(event) {
                        lume.on_key_event(key_event)
                    }
                }
            }
            WindowEvent::Resized(size) => {
                if size.width > 0 && size.height > 0 {
                    if let Some(lume) = self.lume.as_mut() {
                        lume.on_update_surface()
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(lume) = self.lume.as_mut() {
                    if let Err(e) = lume.draw() {
                        error!("Failed to draw frame: {:?}", e);
                    }
                }
            }
            WindowEvent::CloseRequested => {
                info!("Close requested");

                if let Some(lume) = self.lume.take() {
                    match lume.on_close() {
                        Ok(_) => info!("Window closed successfully"),
                        Err(error) => panic!("Window closed with error: {}", error),
                    }
                }

                event_loop.exit();
            }
            WindowEvent::CursorMoved { position, .. } => {
                let mouse_event = MouseEvent::Move {
                    position: [position.x as f32, position.y as f32],
                };

                if let Some(lume) = self.lume.as_mut() {
                    lume.on_mouse_event(mouse_event)
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let mouse_event = MouseEvent::Click {
                    button: match button {
                        WinitMouseButton::Left => MouseButton::Left,
                        WinitMouseButton::Right => MouseButton::Right,
                        WinitMouseButton::Middle => MouseButton::Middle,
                        WinitMouseButton::Forward => MouseButton::Forward,
                        WinitMouseButton::Back => MouseButton::Back,
                        WinitMouseButton::Other(event) => {
                            warn!("Unexpected mouse event! Code: {}", event);

                            return;
                        }
                    },
                    state: match state {
                        ElementState::Pressed => EventState::Down,
                        ElementState::Released => EventState::Up,
                    }
                };

                if let Some(lume) = self.lume.as_mut() {
                    lume.on_mouse_event(mouse_event)
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let mouse_event = MouseEvent::Scroll {
                    delta: match delta {
                        MouseScrollDelta::LineDelta(lines, rows) => [lines * 15.0, -rows * 15.0],
                        MouseScrollDelta::PixelDelta(position) => [
                            position.x as f32,
                            position.y as f32,
                        ]
                    }
                };

                if let Some(lume) = self.lume.as_mut() {
                    lume.on_mouse_event(mouse_event)
                }
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
