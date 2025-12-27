use crate::platform_providers::io_provider::IOProvider;
use crate::platform_providers::surface_provider::SurfaceProvider;
use std::sync::Arc;

pub struct Providers {
    pub io_provider: Arc<dyn IOProvider>,
    pub surface_provider: Arc<dyn SurfaceProvider>,
}
