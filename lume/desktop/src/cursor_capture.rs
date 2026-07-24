use tracing::warn;
use winit::dpi::PhysicalPosition;
use winit::window::{CursorGrabMode, Window};

pub struct CursorCapture {
    anchor: PhysicalPosition<f64>,

    active: bool,
    motion_enabled: bool,
}

impl CursorCapture {
    pub fn new() -> Self {
        Self {
            anchor: PhysicalPosition::new(0.0, 0.0),

            active: false,
            motion_enabled: false,
        }
    }

    pub fn set_active(&mut self, window: &Window, active: bool) {
        if active == self.active {
            return;
        }

        if active {
            window.set_cursor_grab(CursorGrabMode::Locked)
                .or_else(|_| window.set_cursor_grab(CursorGrabMode::Confined))
                .unwrap_or_else(|error| warn!("Failed to grab cursor: {}", error));
        } else {
            let _ = window.set_cursor_grab(CursorGrabMode::None);
            let _ = window.set_cursor_position(self.anchor);

            self.motion_enabled = false;
        }

        window.set_cursor_visible(!active);

        self.active = active;
    }

    pub fn observe_cursor(
        &mut self,
        window: &Window,
        position: PhysicalPosition<f64>,
    ) -> Option<PhysicalPosition<f64>> {
        if !self.active {
            self.anchor = position;

            return Some(position);
        }

        if position != self.anchor {
            let _ = window.set_cursor_position(self.anchor);
        }

        None
    }

    pub fn end_frame(&mut self) {
        self.motion_enabled = self.active;
    }

    pub fn accepts_motion(&self) -> bool {
        self.motion_enabled
    }
}
