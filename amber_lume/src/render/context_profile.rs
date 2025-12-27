use crate::platform_providers::surface_provider::SurfaceProvider;
use anyhow::{Context, Result};
use ash_window::enumerate_required_extensions;
use std::ffi::CStr;
use std::sync::Arc;
use tracing::debug;

pub struct ContextProfile<'a> {
    pub extensions: &'a [*const i8],

    pub enable_validation: bool,
}

impl<'a> ContextProfile<'a> {
    pub fn from(surface_provider: Arc<dyn SurfaceProvider>) -> Result<Self> {
        let (raw_display_handle, _) = surface_provider.handles();
        let extensions = enumerate_required_extensions(raw_display_handle)
            .context("enumerate_required_extensions")?;

        debug!(
            "Required instance extensions: {:?}",
            Self::display_ext_names(extensions)
        );

        Ok(Self {
            extensions,

            enable_validation: true,
        })
    }

    fn display_ext_names(extensions: &[*const i8]) -> Vec<String> {
        extensions
            .iter()
            .map(|&p| unsafe { CStr::from_ptr(p) }.to_string_lossy().into_owned())
            .collect()
    }
}
