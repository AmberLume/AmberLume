use anyhow::{Context, Result};
use ash_window::enumerate_required_extensions;
use raw_window_handle::RawDisplayHandle;
use std::ffi::{c_char, CStr};
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
        display_handle: RawDisplayHandle,
        layers: Vec<VulkanLayer>,
        validation_features: Vec<ValidationFeatures>,
    ) -> Result<Self> {
        let extensions = enumerate_required_extensions(display_handle)
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
