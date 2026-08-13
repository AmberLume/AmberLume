use resource_reader::IOProvider;
use gpu::SurfaceProvider;
use std::sync::Arc;

pub struct Providers {
    pub io_provider: Arc<dyn IOProvider>,
    pub surface_provider: Arc<dyn SurfaceProvider>,
}
