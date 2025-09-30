use anyhow::Context;
use anyhow::Result;
use ash::vk::{ApplicationInfo, InstanceCreateInfo, make_api_version};
use ash::{Entry, Instance, vk};
use ash_window::{create_surface, enumerate_required_extensions};
use std::ffi::CStr;
use tracing::{debug, info, instrument};
use vk::SurfaceKHR;
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use winit::window::Window;

pub struct InstanceSurface {
    pub instance: Instance,
    pub surface_loader: ash::khr::surface::Instance,
    pub surface: SurfaceKHR,
}

impl InstanceSurface {
    #[instrument(level = "trace", skip(entry, window))]
    pub fn create(entry: &Entry, window: &Window) -> Result<Self> {
        let raw_display = window.display_handle()?.as_raw();
        let required_extensions =
            enumerate_required_extensions(raw_display).context("enumerate_required_extensions")?;
        debug!(
            "Required instance extensions: {:?}",
            Self::display_ext_names(required_extensions)
        );

        let app_name = CStr::from_bytes_with_nul(b"Ebb\0")?;
        let app_version = make_api_version(0, 0, 1, 0);
        let engine_name = CStr::from_bytes_with_nul(b"AmberLume\0")?;
        let engine_version = make_api_version(0, 0, 1, 0);
        let app_info = ApplicationInfo::default()
            .application_name(app_name)
            .application_version(app_version)
            .engine_name(engine_name)
            .engine_version(engine_version)
            .api_version(vk::API_VERSION_1_3);

        let extension_names: Vec<*const i8> = required_extensions.iter().map(|&e| e).collect();
        let instance_create_info = InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_extension_names(&extension_names);

        let instance = unsafe { entry.create_instance(&instance_create_info, None) }
            .context("create_instance")?;
        let surface_loader = ash::khr::surface::Instance::new(entry, &instance);

        let raw_window_handle = window.window_handle()?.as_raw();
        let surface =
            unsafe { create_surface(entry, &instance, raw_display, raw_window_handle, None) }
                .context("create_surface")?;

        let instance_surface = Self {
            instance,
            surface_loader,
            surface,
        };

        info!("InstanceSurface created");

        Ok(instance_surface)
    }

    pub fn destroy(&self) {
        unsafe {
            self.surface_loader.destroy_surface(self.surface, None);
            self.instance.destroy_instance(None);
        }
        info!("InstanceSurface destroyed");
    }

    fn display_ext_names(extensions: &[*const i8]) -> Vec<String> {
        extensions
            .iter()
            .map(|&p| unsafe { CStr::from_ptr(p) }.to_string_lossy().into_owned())
            .collect()
    }
}
