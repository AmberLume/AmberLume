use crate::platform_providers::surface_provider::VulkanSurfaceProvider;
use amber_lume::input_handler::hardware_key_codes::HardwareKeyCode;
use anyhow::{bail, Result};
use core::lume::Lume;
use std::sync::Arc;
use tracing::{error, info, instrument, trace, warn};
use winit::application::ApplicationHandler;
use winit::event::{
    DeviceEvent, DeviceId, ElementState, KeyEvent as WinitKeyEvent,
    MouseButton as WinitMouseButton, MouseScrollDelta, WindowEvent,
};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorGrabMode, Window, WindowAttributes, WindowId};
use amber_lume::input_handler::hardware_pointer_event::HardwarePointerEvent;
use amber_lume::input_handler::hardware_pointer_key_codes::HardwarePointerKeyCodes;
use amber_lume::input_handler::input_frame::PointerId;
use amber_lume::lifecycle::lifecycle::AmberLumeLifecycle;

pub struct Application {
    attributes: WindowAttributes,

    window: Option<Arc<Window>>,

    lume: Lume,
}

impl Application {
    pub fn new(attributes: WindowAttributes, lume: Lume) -> Self {
        trace!("Creating Application...");

        Self {
            attributes,

            window: None,

            lume,
        }
    }

    fn winit_to_hardware(event: WinitKeyEvent) -> Result<(HardwareKeyCode, bool)> {
        let keycode = if let PhysicalKey::Code(code) = event.physical_key {
            match code {
                KeyCode::Escape => HardwareKeyCode::Esc,
                KeyCode::F1 => HardwareKeyCode::F1,
                KeyCode::F2 => HardwareKeyCode::F2,
                KeyCode::F3 => HardwareKeyCode::F3,
                KeyCode::F4 => HardwareKeyCode::F4,
                KeyCode::F5 => HardwareKeyCode::F5,
                KeyCode::F6 => HardwareKeyCode::F6,
                KeyCode::F7 => HardwareKeyCode::F7,
                KeyCode::F8 => HardwareKeyCode::F8,
                KeyCode::F9 => HardwareKeyCode::F9,
                KeyCode::F10 => HardwareKeyCode::F10,
                KeyCode::F11 => HardwareKeyCode::F11,
                KeyCode::F12 => HardwareKeyCode::F12,
                KeyCode::KeyQ => HardwareKeyCode::Q,
                KeyCode::KeyW => HardwareKeyCode::W,
                KeyCode::KeyE => HardwareKeyCode::E,
                KeyCode::KeyR => HardwareKeyCode::R,
                KeyCode::KeyT => HardwareKeyCode::T,
                KeyCode::KeyY => HardwareKeyCode::Y,
                KeyCode::KeyU => HardwareKeyCode::U,
                KeyCode::KeyI => HardwareKeyCode::I,
                KeyCode::KeyO => HardwareKeyCode::O,
                KeyCode::KeyP => HardwareKeyCode::P,
                KeyCode::KeyA => HardwareKeyCode::A,
                KeyCode::KeyS => HardwareKeyCode::S,
                KeyCode::KeyD => HardwareKeyCode::D,
                KeyCode::KeyF => HardwareKeyCode::F,
                KeyCode::KeyG => HardwareKeyCode::G,
                KeyCode::KeyH => HardwareKeyCode::H,
                KeyCode::KeyJ => HardwareKeyCode::J,
                KeyCode::KeyK => HardwareKeyCode::K,
                KeyCode::KeyL => HardwareKeyCode::L,
                KeyCode::KeyZ => HardwareKeyCode::Z,
                KeyCode::KeyX => HardwareKeyCode::X,
                KeyCode::KeyC => HardwareKeyCode::C,
                KeyCode::KeyV => HardwareKeyCode::V,
                KeyCode::KeyB => HardwareKeyCode::B,
                KeyCode::KeyN => HardwareKeyCode::N,
                KeyCode::KeyM => HardwareKeyCode::M,
                KeyCode::ArrowUp => HardwareKeyCode::ArrowUp,
                KeyCode::ArrowLeft => HardwareKeyCode::ArrowLeft,
                KeyCode::ArrowDown => HardwareKeyCode::ArrowDown,
                KeyCode::ArrowRight => HardwareKeyCode::ArrowRight,
                KeyCode::Space => HardwareKeyCode::Space,
                KeyCode::ShiftLeft | KeyCode::ShiftRight => HardwareKeyCode::Shift,
                _ => bail!("Received unhandled keycode: {:?}", event.physical_key),
            }
        } else {
            bail!(
                "Received physical key is not a code: {:?}",
                event.physical_key
            );
        };

        let state = match event.state {
            ElementState::Pressed => true,
            ElementState::Released => false,
        };

        Ok((keycode, state))
    }
    
    pub fn destroy(self) -> Result<()> {
        self.lume.destroy()?;
        
        Ok(())
    }
}

impl ApplicationHandler for Application {
    #[instrument(level = "trace", skip_all)]
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(event_loop.create_window(self.attributes.clone()).unwrap());

        trace!("Window created");

        let surface_provider = Arc::new(VulkanSurfaceProvider::new(window.clone()));

        let target = match self.lume.create_surface_target(surface_provider) {
            Ok(target) => target,
            Err(e) => {
                error!("Failed to create surface target: {}", e);

                event_loop.exit();
                return;
            }
        };

        match self.lume.attach_render_target(target) {
            Ok(()) => {
                self.window = Some(window);
            }
            Err(e) => {
                error!("Failed to attach render target: {}", e);

                event_loop.exit();
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

                if let Ok((keycode, state)) = Self::winit_to_hardware(event) {
                    self.lume.push_hardware_keycode_event(keycode, state)
                }
            }
            WindowEvent::Resized(size) => {
                if size.width > 0 && size.height > 0 {
                    self.lume.on_update_surface();
                }
            }
            WindowEvent::RedrawRequested => {
                if let Err(e) = self.lume.draw() {
                    panic!("Failed to draw frame: {:?}", e);
                }
            }
            WindowEvent::CloseRequested => {
                info!("Close requested");

                match self.lume.detach_render_target() {
                    Ok(_) => info!("Window closed successfully"),
                    Err(error) => panic!("Window closed with error: {}", error),
                }

                event_loop.exit();
            }
            WindowEvent::CursorMoved { position, .. } => {
                let position = (position.x as f32, position.y as f32);
                let pointer_id = PointerId::new(0);

                self.lume.push_hardware_pointer_event(&pointer_id, HardwarePointerEvent::Move {
                    position,
                });
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let pointer_id = PointerId::new(0);

                let button = match button {
                    WinitMouseButton::Left => HardwarePointerKeyCodes::Left,
                    WinitMouseButton::Right => HardwarePointerKeyCodes::Right,
                    WinitMouseButton::Middle => HardwarePointerKeyCodes::Middle,
                    WinitMouseButton::Forward => HardwarePointerKeyCodes::Forward,
                    WinitMouseButton::Back => HardwarePointerKeyCodes::Backward,
                    WinitMouseButton::Other(event) => {
                        warn!("Unexpected mouse event! Code: {}", event);

                        return;
                    }
                };
                let pressed = match state {
                    ElementState::Pressed => true,
                    ElementState::Released => false,
                };

                self.lume.push_hardware_pointer_event(&pointer_id, HardwarePointerEvent::Button {
                    button,
                    pressed,
                });
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let pointer_id = PointerId::new(0);

                let delta = match delta {
                    MouseScrollDelta::LineDelta(lines, rows) => (lines * 16.0, -rows * 16.0),
                    MouseScrollDelta::PixelDelta(position) => {
                        (position.x as f32, position.y as f32)
                    }
                };

                self.lume.push_hardware_pointer_event(&pointer_id, HardwarePointerEvent::Scroll { delta });
            }
            _ => {}
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        let DeviceEvent::MouseMotion { delta } = event else {
            return;
        };

        if !self.lume.engine_settings().get_current().load().input.cursor_controls_camera.value {
            return;
        }

        self.lume.push_hardware_pointer_event(&PointerId::new(0), HardwarePointerEvent::Motion {
            delta: (delta.0 as f32, delta.1 as f32),
        });
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        trace!("About to wait called");

        let Some(window) = self.window.as_ref() else {
            return;
        };

        let settings = self.lume.engine_settings().get_current().load();

        if settings.input.cursor_controls_camera.value {
            window.set_cursor_grab(CursorGrabMode::Locked)
                .or_else(|_| window.set_cursor_grab(CursorGrabMode::Confined))
                .unwrap_or_else(|e| warn!("Failed to grab cursor: {}", e));

            window.set_cursor_visible(false);
        } else {
            let _ = window.set_cursor_grab(CursorGrabMode::None);

            window.set_cursor_visible(true);
        }

        window.request_redraw();
    }
}
