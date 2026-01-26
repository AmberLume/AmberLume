use crate::desktop_application::Application;
use crate::tracing::Tracing;
use winit::dpi::{PhysicalSize, Size};
use winit::event_loop::EventLoop;
#[cfg(feature = "x11")]
use winit::platform::x11::EventLoopBuilderExtX11;
use winit::window::WindowAttributes;

mod desktop_application;
mod engine;
mod lume;
mod platform_providers;
pub mod scene;
mod tracing;
mod ui;

fn main() {
    Tracing::initialize();

    let size = Size::Physical(PhysicalSize::new(1280, 720));
    let config = WindowAttributes::default()
        .with_title(String::from("AmberLume"))
        .with_inner_size(size)
        .with_resizable(true);

    let mut application = Application::new(config);

    let event_loop = create_event_loop();
    event_loop.run_app(&mut application).unwrap();
}

fn create_event_loop() -> EventLoop<()> {
    #[cfg(feature = "x11")]
    let event_loop = EventLoop::builder().with_x11().build().unwrap();

    #[cfg(not(feature = "x11"))]
    let event_loop = EventLoop::builder().build().unwrap();

    event_loop
}
