use crate::platform_providers::surface_provider::SurfaceProvider;
use anyhow::{Context, Result};
use ash_window::enumerate_required_extensions;
use std::ffi::{c_char, CStr};
use std::sync::Arc;
use tracing::debug;
use crate::render::device::layers::VulkanLayer;
use crate::render::device::validation_features::ValidationFeatures;

pub struct ContextProfile<'a> {
    pub extensions: &'a [*const c_char],
    pub layers: Vec<VulkanLayer>,
    pub validation_features: Vec<ValidationFeatures>,
}

impl<'a> ContextProfile<'a> {
    pub fn from(
        surface_provider: Arc<dyn SurfaceProvider>,
        layers: Vec<VulkanLayer>,
        validation_features: Vec<ValidationFeatures>,
    ) -> Result<Self> {
        let (raw_display_handle, _) = surface_provider.handles();
        let extensions = enumerate_required_extensions(raw_display_handle)
            .context("enumerate_required_extensions")?;

        debug!(
            "Required instance extensions: {:?}",
            Self::display_ext_names(extensions)
        );

        Ok(Self {
            extensions,
            layers,
            validation_features,
        })
    }

    fn display_ext_names(extensions: &[*const c_char]) -> Vec<String> {
        extensions
            .iter()
            .map(|&p| unsafe { CStr::from_ptr(p) }.to_string_lossy().into_owned())
            .collect()
    }
}
