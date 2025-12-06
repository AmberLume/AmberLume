use crate::render::vulkan::surface::surface_provider::SurfaceProvider;
use crate::resources::io_provider::IOProvider;
use std::sync::Arc;

pub struct Providers {
    pub io_provider: Arc<dyn IOProvider>,
    pub surface_provider: Arc<dyn SurfaceProvider>,
}
