use anyhow::{Context, Result};
use ash_window::enumerate_required_extensions;
use std::ffi::CStr;
use tracing::debug;
use winit::raw_window_handle::HasDisplayHandle;
use winit::window::Window;

pub struct ContextProfile<'a> {
    pub extensions: &'a [*const i8],
}

impl<'a> ContextProfile<'a> {
    pub fn from(window: &Window) -> Result<Self> {
        let raw_display = window.display_handle()?.as_raw();
        let extensions =
            enumerate_required_extensions(raw_display).context("enumerate_required_extensions")?;

        debug!(
            "Required instance extensions: {:?}",
            Self::display_ext_names(extensions)
        );

        Ok(Self { extensions })
    }

    fn display_ext_names(extensions: &[*const i8]) -> Vec<String> {
        extensions
            .iter()
            .map(|&p| unsafe { CStr::from_ptr(p) }.to_string_lossy().into_owned())
            .collect()
    }
}
