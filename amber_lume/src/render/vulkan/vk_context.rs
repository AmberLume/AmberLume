use crate::render::context_profile::ContextProfile;
use crate::render::vulkan::physical_device_info::PhysicalDeviceInfo;
use anyhow::{Context, Result};
use ash::khr::surface::Instance as SurfaceLoader;
use ash::vk::{ApplicationInfo, InstanceCreateInfo, make_api_version};
use ash::{Entry, Instance, vk};
use std::ffi::CStr;
use tracing::info;

pub struct VkContext {
    pub entry: Entry,

    pub instance: Instance,
    pub surface_loader: SurfaceLoader,

    pub physical_devices: Vec<PhysicalDeviceInfo>,
}

impl VkContext {
    pub fn new(context_profile: ContextProfile) -> Result<Self> {
        let entry = Entry::linked();

        let instance = Self::create_instance(&entry, context_profile)?;
        let surface_loader = Self::create_surface_loader(&entry, &instance)?;

        let physical_devices = PhysicalDeviceInfo::create_all(&instance)?;

        info!("VkContext is ready");

        let vk_context = Self {
            entry,

            instance,
            surface_loader,

            physical_devices,
        };

        Ok(vk_context)
    }

    fn create_instance(entry: &Entry, context_profile: ContextProfile) -> Result<Instance> {
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

        let extension_names: Vec<*const i8> =
            context_profile.extensions.iter().map(|&e| e).collect();
        let instance_create_info = InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_extension_names(&extension_names);

        let instance = unsafe { entry.create_instance(&instance_create_info, None) }
            .context("create_instance")?;

        Ok(instance)
    }

    fn create_surface_loader(entry: &Entry, instance: &Instance) -> Result<SurfaceLoader> {
        let surface_loader = SurfaceLoader::new(&entry, &instance);

        Ok(surface_loader)
    }
}

impl Drop for VkContext {
    fn drop(&mut self) {}
}
