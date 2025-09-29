use winit::dpi::{PhysicalSize, Size};
use winit::event_loop::EventLoop;
use winit::window::WindowAttributes;
use crate::application::Application;
use crate::tracing::Tracing;

mod application;
mod tracing;
mod vulkan;

fn main() {
    Tracing::initialize();

    let size = Size::Physical(PhysicalSize::new(1280, 720));
    let config = WindowAttributes::default()
        .with_title(String::from( "AmberLume"))
        .with_inner_size(size)
        .with_resizable(true);

    let mut application = Application::new(config);

    let event_loop = EventLoop::new().unwrap();
    event_loop.run_app(&mut application).unwrap();
}
